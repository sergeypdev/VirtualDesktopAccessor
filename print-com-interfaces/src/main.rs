//! This program uses [Microsoft Symbol Server] to get debug symbols for
//! `twinui.pcshell.dll` and then searches those symbols for information related
//! to the Virtual Desktop COM interfaces.
//!
//! Code was inspired by the python script at [GetVirtualDesktopAPI_DIA]
//!
//! [GetVirtualDesktopAPI_DIA]: https://github.com/mzomparelli/GetVirtualDesktopAPI_DIA
//! [Microsoft Symbol Server]: https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/microsoft-public-symbols

use debugid::DebugId;
use eyre::{Context, OptionExt};
use object::{
    read::pe::{ImageNtHeaders, ImageOptionalHeader, PeFile32, PeFile64},
    Object as _,
};
use pdb::{FallibleIterator, Rva, PDB};
use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};
use symbolic_demangle::Demangle as _;
use symsrv::SymsrvDownloader;

const TWINUI_PCSHELL_PATH: &str = r"C:\Windows\System32\twinui.pcshell.dll";

// This dll doesn't seem to have any relevant symbols, but it was searched by
// the original Python script:
// const ACTXPRXY_PATH: &str = r"C:\Windows\System32\actxprxy.dll";

/// Parts of known mangled names for vtables
const VIRTUAL_DESKTOP_V_TABLE_NAMES: &[&str] = &[
    "??_7CVirtualDesktop@@6BIVirtualDesktop@@@",
    "??_7CVirtualDesktopManager@@6B?$ImplementsHelper@U?$RuntimeClassFlags@$02@WRL@Microsoft@@$00UIVirtualDesktopManagerInternal@@UISuspendableVirtualDesktopManager@@VFtmBase@23@@Details@WRL@Microsoft@@@",
    "??_7CVirtualDesktopNotificationsDerived@@6BIVirtualDesktopNotification@@@",
    "??_7CVirtualDesktopNotificationsDerived@@6B@",
    "??_7CVirtualDesktopHotkeyHandler@@6B@",
    "??_7CVirtualDesktopHotkeyHandler@@6B@",
    "??_7VirtualDesktopsAp",
    "??_7VirtualPinnedAppsHandler@@6B?$Chain",
];

/// The code ID for a Windows PE file.
///
/// When combined with the binary name, the `PeCodeId` lets you obtain binaries from
/// symbol servers. It is not useful on its own, it has to be paired with the binary name.
///
/// A Windows binary's `PeCodeId` is distinct from its debug ID (= pdb GUID + age).
/// If you have a binary file, you can get both the `PeCodeId` and the debug ID
/// from it. If you only have a PDB file, you usually *cannot* get the `PeCodeId` of
/// the corresponding binary from it.
///
/// Note: copied from the [`wholesym`] crate.
///
/// [`wholesym`]: https://docs.rs/samply-symbols/0.23.0/src/samply_symbols/shared.rs.html#227
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PeCodeId {
    pub timestamp: u32,
    pub image_size: u32,
}
impl PeCodeId {
    /// Code from `make_library_info` function in "samply-symbols-0.23.0\src\binary_image.rs".
    pub fn for_file_data(data: &[u8]) -> eyre::Result<Self> {
        if let Ok(file) = PeFile64::parse(data) {
            Self::from_pe_file(file)
        } else {
            Self::from_pe_file(PeFile32::parse(data)?)
        }
    }
    /// Code from pe_info function in "samply-symbols-0.23.0\src\binary_image.rs"
    fn from_pe_file<'buf, Pe: ImageNtHeaders>(
        pe: object::read::pe::PeFile<'buf, Pe, &'buf [u8]>,
    ) -> eyre::Result<Self> {
        // The code identifier consists of the `time_date_stamp` field id the COFF header, followed by
        // the `size_of_image` field in the optional header. If the optional PE header is not present,
        // this identifier is `None`.
        let header = pe.nt_headers();
        let timestamp = header
            .file_header()
            .time_date_stamp
            .get(object::LittleEndian);
        let image_size = header.optional_header().size_of_image();
        Ok(PeCodeId {
            timestamp,
            image_size,
        })
    }
}
impl FromStr for PeCodeId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 9 || s.len() > 16 {
            return Err(());
        }
        let timestamp = u32::from_str_radix(&s[..8], 16).map_err(|_| ())?;
        let image_size = u32::from_str_radix(&s[8..], 16).map_err(|_| ())?;
        Ok(Self {
            timestamp,
            image_size,
        })
    }
}
impl std::fmt::Display for PeCodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:08X}{:x}", self.timestamp, self.image_size))
    }
}

/// The Portable Executable (PE) format is a file format for executables, object
/// code, DLLs.
pub struct PeFile {
    /// Path where the dll file can be found.
    dll_path: PathBuf,
    /// Path where debug info can be found.
    pdb_path: Option<PathBuf>,
}
impl PeFile {
    pub fn new(dll_path: impl Into<PathBuf>) -> Self {
        Self {
            dll_path: dll_path.into(),
            pdb_path: None,
        }
    }
    /// Get a debug id that can be used to download a `.pdb` file. Use the
    /// [`DebugId::breakpad`] method and then [`ToString::to_string`] that.
    pub fn debug_id(&self) -> eyre::Result<DebugId> {
        let data = std::fs::read(&self.dll_path)
            .with_context(|| format!("Failed to read {}", self.dll_path.display()))?;

        let object = object::File::parse(data.as_slice())?;

        if let Ok(Some(pdb_info)) = object.pdb_info() {
            // Copied code from "samply-symbols-0.23.0\src\debugid_util.rs"
            Ok(DebugId::from_guid_age(&pdb_info.guid(), pdb_info.age())?)
        } else {
            Err(eyre::eyre!("No debug info available for object"))
        }
    }
    /// This id can be used to download the `.dll` file.
    pub fn pe_code_id(&self) -> eyre::Result<PeCodeId> {
        let data = std::fs::read(&self.dll_path)
            .with_context(|| format!("Failed to read {}", self.dll_path.display()))?;
        PeCodeId::for_file_data(data.as_slice())
    }
    /// Download and cache `.pdb` debug symbol file.
    pub async fn download_pdb(&mut self, downloader: &SymsrvDownloader) -> eyre::Result<()> {
        let pdb_name = Path::new(&self.dll_path);
        let pdb_name = pdb_name.with_extension("pdb");
        let pdb_name = pdb_name
            .file_name()
            .ok_or_eyre("dll paths have file names")?
            .to_str()
            .ok_or_eyre("dll files have UTF8 file names")?;

        // Get hash:
        let hash = self.debug_id()?.breakpad().to_string();

        // Download and cache a PDB file.
        let local_path = downloader.get_file(pdb_name, &hash).await?;
        self.pdb_path = Some(local_path);
        Ok(())
    }
    pub fn open_pdb(&self) -> eyre::Result<PDB<'static, File>> {
        let file = std::fs::File::open(
            self.pdb_path
                .as_deref()
                .ok_or_eyre("Haven't downloaded pdb file yet")?,
        )?;
        Ok(pdb::PDB::open(file)?)
    }
    pub fn read_dll(&self) -> eyre::Result<Vec<u8>> {
        Ok(std::fs::read(&self.dll_path)?)
    }
}

fn setup_download_next_to_exe() -> SymsrvDownloader {
    // Parse the _NT_SYMBOL_PATH environment variable.
    let symbol_path_env = symsrv::get_symbol_path_from_environment();
    let symbol_path = symbol_path_env
        .as_deref()
        .unwrap_or("srv**https://msdl.microsoft.com/download/symbols");
    let parsed_symbol_path = symsrv::parse_nt_symbol_path(symbol_path);

    // Create a downloader which follows the _NT_SYMBOL_PATH recipe.
    let mut downloader = SymsrvDownloader::new(parsed_symbol_path);
    downloader.set_default_downstream_store(
        // Download files next to the executable:
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|exe_dir| exe_dir.join("Symbols")))
            .or_else(symsrv::get_home_sym_dir),
    );
    downloader
}

#[derive(Debug, Default, Clone, Copy)]
struct AddressInfo {
    size: u32,
    rva: Rva,
}

/// Inspired by [`symbolic_debuginfo::SymbolMap::from_iter`], assumes that a
/// symbol occupies all space until the next symbol.
fn calculate_size_for_symbols(
    symbols: &mut [(Option<AddressInfo>, pdb::Symbol<'_>)],
    address_map: &pdb::AddressMap<'_>,
) {
    let mut symbols = symbols
        .iter_mut()
        .filter_map(|(size, sym)| match sym.parse().ok()? {
            pdb::SymbolData::Public(public_symbol) => public_symbol
                .offset
                .to_rva(address_map)
                .map(|rva| (size, rva)),
            pdb::SymbolData::ProcedureReference(_proc_ref) => {
                // Ignore for now
                None
            }
            unexpected => todo!("didn't expect this kod of symbol: {unexpected:?}"),
        })
        .collect::<Vec<_>>();
    symbols.sort_by_key(|(_, start)| start.0);

    // symbols.dedup_by_key(|(_, start)| start.0); // We could do this but then some symbols won't get a size...

    for ix in 1..symbols.len() {
        let start = symbols[ix - 1].1;
        // There might be multiple "symbols" at the same offset, find the next one:
        let Some((_, end)) = symbols[ix..].iter().find(|(_, end)| *end != start) else {
            break;
        };
        let size = end
            .checked_sub(start)
            .expect("Since symbols are sorted the later once should have larger offsets");
        *symbols[ix - 1].0 = Some(AddressInfo { size, rva: start });
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let mut unfiltered = false;
    for arg in std::env::args().skip(1) {
        if arg.eq_ignore_ascii_case("--all") {
            unfiltered = true;
        } else {
            eyre::bail!("Unknown cli argument: {arg:?}");
        }
    }

    let downloader = setup_download_next_to_exe();

    let mut twinui = PeFile::new(TWINUI_PCSHELL_PATH);
    let mut pe_files = [&mut twinui];

    for pe_file in &mut pe_files {
        pe_file.download_pdb(&downloader).await?;
        println!(
            "Find pdb debug file at: {}",
            pe_file.pdb_path.as_ref().unwrap().display()
        );
    }

    let mut twinui_pdb = twinui.open_pdb()?;

    let twinui_debug = twinui_pdb.debug_information()?;
    let twinui_modules = twinui_debug.modules()?.collect::<Vec<_>>()?;

    let mut twinui_fn_info = twinui.open_pdb()?;
    twinui_fn_info.debug_information()?;

    if !twinui_fn_info.type_information()?.is_empty() {
        eprintln!("Info: Type info isn't empty as was expected, perhaps it could be useful");
    }
    if !twinui_fn_info.frame_table()?.is_empty() {
        eprintln!("Info: Frame table isn't empty as was expected, perhaps it could be useful");
    }
    if !twinui_fn_info.id_information()?.is_empty() {
        eprintln!("Info: Id information isn't empty as was expected, perhaps it could be useful");
    }

    let mut symbol_lookup = HashMap::new();

    let mut total_in_modules = 0;
    for module in twinui_modules.iter() {
        //println!("#{module_index} module name: {}, object file name: {}",module.module_name(),module.object_file_name());
        let info = match twinui_fn_info.module_info(module)? {
            Some(info) => {
                let count = info.symbols()?.count()?;
                total_in_modules += count;
                //println!("\tcontains {count} symbols");
                info
            }
            None => {
                //println!("\tmodule information not available");
                continue;
            }
        };
        let mut syms = info.symbols()?;
        while let Some(sym) = syms.next()? {
            let _parsed = sym.parse()?;
            // println!("\tModule symbol: {parsed:?}");
        }
    }

    let twinui_symbols = twinui_fn_info.global_symbols()?;
    let twinui_address_map = twinui_fn_info.address_map()?;
    twinui_fn_info.debug_information()?;

    let mut all_symbols = twinui_symbols
        .iter()
        .map(|sym| Ok((None, sym)))
        .collect::<Vec<_>>()?;
    calculate_size_for_symbols(all_symbols.as_mut_slice(), &twinui_address_map);
    for (info, sym) in &all_symbols {
        let Some(info) = info else { continue };
        symbol_lookup.insert(info.rva, (info, sym));
    }

    println!(
        "Symbols in modules compared to global: {}/{}",
        total_in_modules,
        all_symbols.len()
    );

    let twinui_data = twinui.read_dll()?;
    let image_base = object::File::parse(twinui_data.as_slice())?.relative_address_base();

    for (size, symbol) in &all_symbols {
        // Will be either SymbolData::ProcedureReference or
        // SymbolData::Public

        if let Ok(pdb::SymbolData::Public(data)) = symbol.parse() {
            // we found the location of a function!
            let rva = data.offset.to_rva(&twinui_address_map).unwrap_or_default();
            let name = data.name.to_string();

            // These filtering rules were ported from the Python script:
            if !(unfiltered
                || (VIRTUAL_DESKTOP_V_TABLE_NAMES
                    .iter()
                    .any(|part| name.contains(part))
                    || (name.contains("_7CWin32ApplicationView")
                        && name.contains("IApplicationView")
                        && !name.contains("Microsoft")
                        && !name.contains("IApplicationViewBase"))))
            {
                // This symbol likely isn't relevant.
                continue;
            }

            let name_info = symbolic_common::Name::new(
                data.name.to_string(),
                symbolic_common::NameMangling::Unknown,
                symbolic_common::Language::Unknown,
            );
            let lang = name_info.detect_language();
            let demangled = name_info.demangle(symbolic_demangle::DemangleOptions::complete());

            if !matches!(&demangled, Some(demangled) if demangled.contains("vftable")) {
                // Not a vtable definition!
                continue;
            }
            println!("\n");
            println!("{} is {}", rva.0, data.name);
            if let Some(name) = &demangled {
                println!("\tDemangeled name ({lang:?}): {name}");
            }
            if let Some(size) = size {
                println!("\tEstimated Size: {}", size.size);
            }
            println!("\t{:?}", data);
            println!();

            let vft_data = &twinui_data[rva.0 as usize..][..size.unwrap_or_default().size as usize];
            let vft_ptrs = vft_data
                .chunks_exact(8)
                .map(|bytes| {
                    u64::from_le_bytes(bytes.try_into().expect("slices should be 8 bytes long"))
                })
                .map(|ptr| ptr.saturating_sub(image_base));
            for (method_index, method_ptr) in vft_ptrs.enumerate() {
                let Ok(method_ptr) = u32::try_from(method_ptr) else {
                    eprintln!(
                        "Warning: a method address in the DLL didn't fit in 32bit and was ignored"
                    );
                    println!("\tMethod {method_index:02}: Unknown ({:x})", method_ptr);
                    continue;
                };
                let method_ptr = Rva(method_ptr);

                if let Some((_info, sym)) = symbol_lookup.get(&method_ptr) {
                    if let Ok(pdb::SymbolData::Public(sym)) = sym.parse() {
                        let name_info = symbolic_common::Name::new(
                            sym.name.to_string(),
                            symbolic_common::NameMangling::Unknown,
                            symbolic_common::Language::Unknown,
                        );
                        let _lang = name_info.detect_language();
                        let demangled =
                            name_info.demangle(symbolic_demangle::DemangleOptions::complete());

                        println!(
                            "\tMethod {method_index:02}: {} ({})",
                            demangled.unwrap_or_default(),
                            sym.name
                        )
                    } else {
                        unreachable!("previously parsed symbol when gather address info");
                    }
                } else {
                    println!("\tMethod {method_index:02}: Unknown ({:x})", method_ptr.0);
                }
            }
        }
    }

    Ok(())
}
