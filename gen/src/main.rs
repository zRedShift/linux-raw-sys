//! A program which generates a linux-headers installation and runs bindgen
//! over the headers, for each supported architecture.

use bindgen::{builder, EnumVariation};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

#[allow(unused_doc_comments)]
const LINUX_VERSION: &str = "v6.17";

/// Some commonly used features.
const DEFAULT_FEATURES: &str = "\"general\", \"errno\"";

/// Plain `#define` ioctl families that use hardcoded numeric values on some
/// architectures instead of `_IO*()` macros.
const PLAIN_IOCTL_INCLUDE: &[&str] = &[
    "FBIO",
    "FIEMAP_",
    "FIO",
    "GIO_",
    "KD",
    "KIOCSOUND",
    "PIO_",
    "PCITEST_LEGACY_IRQ",
    "SIOC",
    "SIOG",
    "TC",
    "TIOC",
    "VT_",
];

/// Non-ioctl constants that share prefixes with the families above.
const PLAIN_IOCTL_EXCLUDE: &[&str] = &[
    "KD_FONT_FLAG",
    "KD_FONT_OP",
    "KD_GRAPHICS",
    "KD_TEXT",
    "TCIO",
    "TCIFL",
    "TCIOF",
    "TCION",
    "TCOF",
    "TCOO",
    "TCSA",
    "TCODE_",
    "TIOCM_",
    "TIOCPKT_",
    "TIOCSER_",
    "VT_ACKACQ",
    "VT_AUTO",
    "VT_EVENT_",
    "VT_MAX_EVENT",
    "VT_PROCESS",
    "VT_SENDSIG",
];

/// Non-ioctl constants produced by function-like macros that appear alongside
/// real `_IO*()` ioctls in the clang_macro_fallback output.
const NON_IOCTL_PREFIX: &[&str] = &[
    "ATM_LM_",
    "ATMLEC_MSG_",
    "BLK_TA_",
    "BLK_TN_",
    "EPOLL",
    "FD_DISK_",
    "FD_NEED_",
    "FD_VERIFY",
    "FMR_OWN_",
    "GSM_FL_",
    "IFF_",
    "INADDR_",
    "KVM_DEV_TYPE_",
    "KVM_DIRTY_GFN_",
    "KVM_EXIT_HYPERCALL_",
    "KVM_IOEVENTFD_FLAG_",
    "KVM_IOEVENTFD_VALID_",
    "KVM_IRQ_ROUTING_XEN_",
    "KVM_PMU_",
    "KVM_TDX_",
    "L2TP_",
    "LIRC_CAN_",
    "OPAL_TABLE_",
    "PERF_BR_ARM64_",
    "PERF_SAMPLE_",
    "PTT_",
    "RWF_",
    "SERIO_",
    "TUN_TAP_",
    "TUN_TUN_",
    "UFFD_API",
    "UFFDIO_CONTINUE_MODE_",
    "UFFDIO_COPY_MODE_",
    "UFFDIO_MOVE_MODE_",
    "UFFDIO_POISON_MODE_",
    "UFFDIO_REGISTER_MODE_",
    "UFFDIO_WRITEPROTECT_MODE_",
    "UFFDIO_ZEROPAGE_MODE_",
    "V4L2_",
    "VESA_",
];

const NON_IOCTL_EXACT: &[&str] = &[
    "AUTOFS_DEV_IOCTL_SIZE",
    "BTRFS_SEARCH_ARGS_BUFSIZE",
    "DMA_HEAP_VALID_FD_FLAGS",
    "FUSE_INVALID_UIDGID",
    "INT_MAX",
    "INT_MIN",
    "JS_RETURN",
    "KCOV_CMP_MASK",
    "MSDOS_DPB",
    "MSDOS_DPS",
    "OPEN_TREE_CLOEXEC",
    "RFKILL_EVENT_SIZE_V1",
    "SNDRV_CTL_VERSION",
    "SNDRV_HWDEP_VERSION",
    "SNDRV_PCM_VERSION",
    "SNDRV_RAWMIDI_VERSION",
    "SNDRV_TIMER_VERSION",
    "TCSADRAIN",
    "TCSAFLUSH",
    "TCSANOW",
    "VC_MAXMSGSIZE",
    "VMMDEVREQ_HGCM_CALL",
];

/// `_IO(0, nr)` ioctls with values < 256 that need an explicit allowlist.
const FALLBACK_IOCTL_ALLOW: &[&str] = &["FIBMAP", "FIGETBSZ"];

fn main() {
    let mut args = env::args();
    let _exe = args.next().unwrap();
    let cmd = args.next();

    // This is the main invocation path.
    assert!(cmd.is_none());
    assert!(args.next().is_none());

    git_init();

    let out = tempfile::TempDir::with_prefix("linux-raw-sys").unwrap();
    let out_dir = out.path();
    let linux_headers = out_dir.join("linux-headers");
    let linux_include = linux_headers.join("include");

    // Clean up any modules from previous builds.
    for entry in fs::read_dir("../src").unwrap() {
        let entry = entry.unwrap();
        assert!(!entry.path().to_str().unwrap().ends_with("."));
        if entry.file_type().unwrap().is_dir() {
            fs::remove_dir_all(entry.path()).ok();
        }
    }

    // Read preambles of lib.rs and Cargo.toml (everything before the
    // auto-generated section). The auto-generated content is buffered
    // and written sorted at the end so that the ioctl-first processing
    // order doesn't affect the output order.
    let src_lib_rs_preamble = {
        let mut s = fs::read_to_string("../src/lib.rs").unwrap();
        let edit_at = s
            .find("// The rest of this file is auto-generated!\n")
            .unwrap();
        s.truncate(edit_at);
        s
    };
    let cargo_toml_preamble = {
        let mut s = fs::read_to_string("../Cargo.toml").unwrap();
        let edit_at = s
            .find("# The rest of this file is auto-generated!\n")
            .unwrap();
        s.truncate(edit_at);
        s
    };

    let mut features: BTreeSet<String> = BTreeSet::new();
    // (arch_index, mod_name, arch_cfg, path)
    let mut lib_rs_entries: Vec<(usize, String, String, String)> = Vec::new();
    let mut arch_index: usize = 0;

    let linux_version = LINUX_VERSION;
    // Checkout a specific version of Linux.
    git_checkout(linux_version);

    let mut linux_archs = fs::read_dir("linux/arch")
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect::<Vec<_>>();
    // Sort archs list as filesystem iteration order is non-deterministic
    linux_archs.sort_by_key(|entry| entry.file_name());
    for linux_arch_entry in linux_archs {
        if !linux_arch_entry.file_type().unwrap().is_dir() {
            continue;
        }
        let linux_arch = linux_arch_entry.file_name().to_str().unwrap().to_owned();

        let rust_arches = rust_arches(&linux_arch);
        if rust_arches.is_empty() {
            continue;
        }

        fs::create_dir_all(&linux_headers).unwrap();

        let mut headers_made = false;
        for rust_arch in rust_arches {
            if !headers_made {
                make_headers_install(&linux_arch, &linux_headers);
                headers_made = true;
            }

            eprintln!(
                "Generating all bindings for Linux {} architecture {}",
                linux_version, rust_arch
            );

            let src_arch = format!("../src/{}", rust_arch);

            fs::create_dir_all(&src_arch).unwrap();

            let mut modules = fs::read_dir("modules")
                .unwrap()
                .map(|entry| entry.unwrap())
                .collect::<Vec<_>>();
            // Sort module list as filesystem iteration order is non-deterministic
            modules.sort_by_key(|entry| entry.file_name());

            // Process ioctl module first so we can derive the blocklist for
            // other modules from its output.
            let mut ioctl_names: HashSet<String> = HashSet::new();
            modules.sort_by_key(|entry| {
                let name = entry.file_name();
                if name.to_str() == Some("ioctl.h") {
                    (0, name)
                } else {
                    (1, name)
                }
            });

            for mod_entry in modules {
                let header_name = mod_entry.path();
                let mod_name = header_name.file_stem().unwrap().to_str().unwrap();
                let mod_rs = format!("{}/{}.rs", src_arch, mod_name);

                let arch_cfg = if *rust_arch == "x32" {
                    "all(target_arch = \"x86_64\", target_pointer_width = \"32\")".to_owned()
                } else if *rust_arch == "x86_64" {
                    "all(target_arch = \"x86_64\", target_pointer_width = \"64\")".to_owned()
                } else {
                    format!("target_arch = \"{}\"", rust_arch)
                };
                lib_rs_entries.push((
                    arch_index,
                    mod_name.to_owned(),
                    arch_cfg,
                    format!("{}/{}.rs", rust_arch, mod_name),
                ));

                run_bindgen(
                    linux_include.to_str().unwrap(),
                    header_name.to_str().unwrap(),
                    &mod_rs,
                    mod_name,
                    rust_arch,
                    linux_version,
                    &ioctl_names,
                );

                if mod_name == "ioctl" {
                    ioctl_names = extract_const_names(&mod_rs);
                }

                features.insert(mod_name.to_owned());
            }

            arch_index += 1;
        }

        fs::remove_dir_all(&linux_headers).unwrap();
    }

    // Write lib.rs sorted by (arch, module) to match alphabetical order.
    lib_rs_entries.sort();
    let mut src_lib_rs = File::create("../src/lib.rs").unwrap();
    src_lib_rs
        .write_all(src_lib_rs_preamble.as_bytes())
        .unwrap();
    src_lib_rs
        .write_all("// The rest of this file is auto-generated!\n".as_bytes())
        .unwrap();
    for (_arch_idx, mod_name, arch_cfg, path) in &lib_rs_entries {
        writeln!(src_lib_rs, "#[cfg(feature = \"{}\")]", mod_name).unwrap();
        writeln!(src_lib_rs, "#[cfg({})]", arch_cfg).unwrap();
        writeln!(src_lib_rs, "#[path = \"{}\"]", path).unwrap();
        writeln!(src_lib_rs, "pub mod {};", mod_name).unwrap();
    }

    // Write Cargo.toml features sorted alphabetically.
    let mut cargo_toml = File::create("../Cargo.toml").unwrap();
    cargo_toml
        .write_all(cargo_toml_preamble.as_bytes())
        .unwrap();
    cargo_toml
        .write_all("# The rest of this file is auto-generated!\n".as_bytes())
        .unwrap();
    writeln!(cargo_toml, "[features]").unwrap();
    for feature in &features {
        writeln!(cargo_toml, "{} = []", feature).unwrap();
    }
    writeln!(cargo_toml, "default = [\"std\", {}]", DEFAULT_FEATURES).unwrap();
    writeln!(cargo_toml, "std = []").unwrap();
    writeln!(cargo_toml, "no_std = []").unwrap();
    writeln!(cargo_toml, "elf = []").unwrap();
    writeln!(cargo_toml, "rustc-dep-of-std = [\"core\", \"no_std\"]").unwrap();

    eprintln!("All bindings generated!");
}

fn git_init() {
    // Clone the linux kernel source repo if necessary. Ignore exit code as it will
    // be non-zero in case it was already cloned.
    //
    // Use a treeless partial clone to save disk space and clone time.
    // See <https://github.blog/2020-12-21-get-up-to-speed-with-partial-clone-and-shallow-clone/>
    // for more info on partial clones.
    //
    // Note: this is not using the official repo
    // <git://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git> but
    // the github fork as the server of the official repo doesn't recognize
    // filtering.
    if !Path::new("linux/.git").exists() {
        assert!(Command::new("git")
            .arg("clone")
            .arg("https://github.com/torvalds/linux.git")
            .arg("--filter=tree:0")
            .arg("--no-checkout")
            .status()
            .unwrap()
            .success());
    }

    // Setup sparse checkout. This greatly reduces the amount of objects necessary
    // to checkout the tree.
    assert!(Command::new("git")
        .arg("sparse-checkout")
        .arg("init")
        .current_dir("linux")
        .status()
        .unwrap()
        .success());

    fs::write(
        "linux/.git/info/sparse-checkout",
        "/*
!/*/
/include/
/arch/
/scripts/
/tools/",
    )
    .unwrap();
}

fn git_checkout(rev: &str) {
    // Delete any generated files from previous versions.
    assert!(Command::new("git")
        .arg("clean")
        .arg("-f")
        .arg("-d")
        .current_dir("linux")
        .status()
        .unwrap()
        .success());

    // Check out the given revision.
    assert!(Command::new("git")
        .arg("checkout")
        .arg(rev)
        .arg("-f")
        .current_dir("linux")
        .status()
        .unwrap()
        .success());

    // Delete any untracked generated files from previous versions.
    assert!(Command::new("git")
        .arg("clean")
        .arg("-f")
        .arg("-d")
        .current_dir("linux")
        .status()
        .unwrap()
        .success());
}

fn make_headers_install(linux_arch: &str, linux_headers: &Path) {
    assert!(Command::new("make")
        .arg("headers_install")
        .arg(format!("ARCH={}", linux_arch))
        .arg(format!(
            "INSTALL_HDR_PATH={}",
            fs::canonicalize(linux_headers).unwrap().to_str().unwrap()
        ))
        .current_dir("linux")
        .status()
        .unwrap()
        .success());

    // HACK: Header missing from kernel installation - likely an upstream bug that needs to be reported
    if linux_arch == "arm64" {
        fs::copy(
            "linux/arch/arm64/include/asm/image.h",
            linux_headers.join("include/asm/image.h"),
        )
        .expect("Missing headers");
    }
}

fn rust_arches(linux_arch: &str) -> &[&str] {
    match linux_arch {
        "arm" => &["arm"],
        "arm64" => &["aarch64"],
        "avr32" => &["avr"],
        "csky" => &["csky"],
        "hexagon" => &["hexagon"],
        "loongarch" => &["loongarch64"],
        "m68k" => &["m68k"],
        "mips" => &["mips", "mips64", "mips32r6", "mips64r6"],
        "powerpc" => &["powerpc", "powerpc64"],
        "riscv" => &["riscv32", "riscv64"],
        "s390" => &["s390x"],
        "sparc" => &["sparc", "sparc64"],
        "x86" => &["x86", "x86_64", "x32"],
        "alpha" | "cris" | "h8300" | "microblaze" | "mn10300" | "score" | "blackfin" | "frv"
        | "ia64" | "m32r" | "m68knommu" | "parisc" | "sh" | "um" | "xtensa" | "unicore32"
        | "c6x" | "nios2" | "openrisc" | "arc" | "nds32" | "metag" | "tile" => &[],
        _ => panic!("unrecognized arch: {}", linux_arch),
    }
}

/// Creates the base bindgen builder with settings common to all modules.
fn make_base_builder(
    linux_include: &str,
    header_name: &str,
    clang_target: &str,
) -> bindgen::Builder {
    let mut builder = builder()
        .rustfmt_configuration_file(Some(Path::new("bindgen-rustfmt.toml").to_owned()))
        .layout_tests(false)
        .generate_comments(false)
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: true,
        })
        .array_pointers_in_arguments(true)
        .derive_debug(true)
        .sort_semantically(true)
        .blocklist_item("^__UAPI_DEF_.*")
        .blocklist_item("BITS_PER_LONG")
        .blocklist_item("__BITS_PER_LONG")
        .clang_arg(format!("--target={}", clang_target))
        .clang_arg("-DBITS_PER_LONG=(__SIZEOF_LONG__*__CHAR_BIT__)")
        .clang_arg("-D__WANT_POSIX1B_SIGNALS__")
        .clang_arg("-nostdinc")
        .clang_arg("-I")
        .clang_arg(linux_include)
        .clang_arg("-I")
        .clang_arg("include")
        .blocklist_item("NULL")
        .use_core()
        .ctypes_prefix("crate::ctypes")
        .header(header_name);

    if clang_target.starts_with("m68k-") {
        // GCC's Linux m68k ABI aligns plain __u64/__s64 to 2 bytes unless
        // the UAPI uses explicit aligned types. Clang's m68k target reports
        // larger unsigned long long alignment, so use a preinclude that
        // preserves explicit __aligned_u64 while matching GCC for plain
        // int-ll64 typedefs.
        builder = builder
            .clang_arg("-include")
            .clang_arg("include/m68k-ioctl-abi.h");
    }

    builder
}

fn run_bindgen(
    linux_include: &str,
    header_name: &str,
    mod_rs: &str,
    mod_name: &str,
    rust_arch: &str,
    linux_version: &str,
    ioctl_names: &HashSet<String>,
) {
    let clang_target = compute_clang_target(rust_arch);

    eprintln!(
        "Generating bindings for {} on Linux {} architecture {}",
        mod_name, linux_version, rust_arch
    );

    if mod_name == "ioctl" {
        // Two-pass: run bindgen with and without clang_macro_fallback to
        // identify which constants come from function-like macros.
        let without_path = format!("{}.no_fb", mod_rs);
        make_ioctl_builder(linux_include, header_name, &clang_target, rust_arch)
            .generate()
            .expect("generate ioctl bindings without fallback")
            .write_to_file(&without_path)
            .expect("write ioctl bindings without fallback");

        make_ioctl_builder(linux_include, header_name, &clang_target, rust_arch)
            .clang_macro_fallback()
            .generate()
            .expect("generate ioctl bindings with fallback")
            .write_to_file(mod_rs)
            .expect("write ioctl bindings with fallback");

        filter_ioctl_two_pass(mod_rs, &without_path);
        fs::remove_file(&without_path).ok();
        return;
    }

    // Non-ioctl modules.
    let mut builder = make_base_builder(linux_include, header_name, &clang_target);

    // Avoid duplicating ioctl constants across multiple modules.
    for name in ioctl_names {
        builder = builder.blocklist_item(name);
    }
    if mod_name != "general" {
        builder = builder.blocklist_item("^LINUX_VERSION_.*");
        builder = builder.blocklist_item("__kernel_fd_set");
        builder = builder.blocklist_item("fds_bits");
        builder = builder.blocklist_item("__FD_SETSIZE");
        builder = builder.blocklist_item("__kernel_sighandler_t");
        builder = builder.blocklist_item("^F_.*");
        builder = builder.blocklist_item("^O_.*");
        builder = builder.blocklist_item("__kernel_fsid_t");
        builder = builder.blocklist_item("^HUGETLB_FLAG_ENCODE_.*");
        builder = builder.blocklist_item("^RESOLVE_.*");
    }
    if rust_arch == "m68k" {
        // Don't emit the `size_t` workaround types for m68k. This should be
        // removed when there's a bindgen release with a fix for [1].
        //
        // [1]: https://github.com/rust-lang/rust-bindgen/issues/3312
        builder = builder.blocklist_type("^s?size_t$");
    }
    builder = builder.clang_macro_fallback();
    if mod_name == "bluetooth" {
        builder = builder.blocklist_type("^sk_buff$");
    }

    let bindings = builder
        .generate()
        .unwrap_or_else(|_| panic!("generate bindings for {}", mod_name));
    let mut bindings = bindings_to_string(&bindings);
    if rust_arch == "m68k" {
        bindings = fix_m68k_plain_u64_layout(&bindings);
    }

    fs::write(mod_rs, dedupe_generated_consts(&bindings))
        .unwrap_or_else(|_| panic!("write_to_file for {}", mod_name));
}

fn make_ioctl_builder(
    linux_include: &str,
    header_name: &str,
    clang_target: &str,
    _rust_arch: &str,
) -> bindgen::Builder {
    make_base_builder(linux_include, header_name, clang_target)
}

fn bindings_to_string(bindings: &bindgen::Bindings) -> String {
    let mut output = Vec::new();
    bindings.write(Box::new(&mut output)).unwrap();
    String::from_utf8(output).unwrap()
}

#[derive(Clone, Copy)]
struct GeneratedConst {
    line_index: usize,
    enum_derived: bool,
}

fn dedupe_generated_consts(bindings: &str) -> String {
    let lines = bindings.lines().collect::<Vec<_>>();
    let mut consts_by_name: HashMap<String, Vec<GeneratedConst>> = HashMap::new();

    for (line_index, line) in lines.iter().enumerate() {
        let Some(name) = generated_const_name(line) else {
            continue;
        };
        consts_by_name
            .entry(name.to_owned())
            .or_default()
            .push(GeneratedConst {
                line_index,
                enum_derived: is_enum_derived_const(line, name),
            });
    }

    let mut drop_indices = HashSet::new();
    for consts in consts_by_name.into_values() {
        if consts.len() <= 1 {
            continue;
        }

        let keep_index = consts
            .iter()
            .find(|generated_const| generated_const.enum_derived)
            .unwrap_or_else(|| consts.last().unwrap())
            .line_index;
        for generated_const in consts {
            if generated_const.line_index != keep_index {
                drop_indices.insert(generated_const.line_index);
            }
        }
    }

    let mut output = String::new();
    for (line_index, line) in lines.iter().enumerate() {
        if !drop_indices.contains(&line_index) {
            output.push_str(line);
            output.push('\n');
        }
    }
    output
}

fn generated_const_name(line: &str) -> Option<&str> {
    line.strip_prefix("pub const ")
        .and_then(|rest| rest.split_once(':'))
        .map(|(name, _rest)| name.trim())
}

fn is_enum_derived_const(line: &str, name: &str) -> bool {
    if !line.contains(": _bindgen_ty_") {
        return false;
    }

    line.split_once('=')
        .map(|(_lhs, rhs)| rhs.trim().trim_end_matches(';').trim())
        .and_then(|rhs| rhs.rsplit_once("::"))
        .is_some_and(|(_enum_ty, variant)| variant == name)
}

#[derive(Debug)]
struct GeneratedField {
    ty: String,
}

#[derive(Debug)]
struct GeneratedItem {
    name: String,
    item_index: usize,
    repr_indices: Vec<usize>,
    has_aligned_repr: bool,
    fields: Vec<GeneratedField>,
}

fn fix_m68k_plain_u64_layout(bindings: &str) -> String {
    let wide_aliases = parse_wide_aliases(bindings);
    let items = parse_generated_items(bindings);
    let pack_types = m68k_pack_types(&items, &wide_aliases);
    apply_m68k_packed_repr(bindings, &items, &pack_types)
}

fn parse_wide_aliases(bindings: &str) -> HashSet<String> {
    let mut aliases = HashMap::new();
    for line in bindings.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("pub type ") {
            let Some((name, ty)) = rest.split_once('=') else {
                continue;
            };
            aliases.insert(
                name.trim().to_owned(),
                ty.trim().trim_end_matches(';').trim().to_owned(),
            );
        }
    }

    let mut wide = HashSet::from([
        "i64".to_owned(),
        "u64".to_owned(),
        "crate::ctypes::c_longlong".to_owned(),
        "crate::ctypes::c_ulonglong".to_owned(),
    ]);

    loop {
        let mut changed = false;
        for (name, ty) in &aliases {
            if wide.contains(name) {
                continue;
            }
            if type_is_wide_scalar(ty, &wide) {
                changed |= wide.insert(name.clone());
            }
        }
        if !changed {
            break;
        }
    }

    wide
}

fn parse_generated_items(bindings: &str) -> Vec<GeneratedItem> {
    let lines = bindings.lines().collect::<Vec<_>>();
    let mut items = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        let item = if let Some(name) = parse_item_name(line, "pub struct ") {
            Some(name)
        } else {
            parse_item_name(line, "pub union ")
        };

        let Some(name) = item else {
            i += 1;
            continue;
        };

        let item_index = i;
        let repr_indices = preceding_repr_indices(&lines, i);
        let has_aligned_repr = repr_indices.iter().any(|idx| lines[*idx].contains("align"));
        let mut fields = Vec::new();
        let mut depth = brace_delta(lines[i]);
        i += 1;
        while i < lines.len() && depth > 0 {
            if let Some(field) = parse_generated_field(lines[i]) {
                fields.push(field);
            }
            depth += brace_delta(lines[i]);
            i += 1;
        }

        items.push(GeneratedItem {
            name,
            item_index,
            repr_indices,
            has_aligned_repr,
            fields,
        });
    }

    items
}

fn parse_item_name(line: &str, prefix: &str) -> Option<String> {
    line.strip_prefix(prefix)
        .and_then(|rest| rest.split_whitespace().next())
        .map(|name| name.trim_end_matches('{').to_owned())
}

fn preceding_repr_indices(lines: &[&str], item_index: usize) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut i = item_index;
    while let Some(prev) = i.checked_sub(1) {
        let line = lines[prev].trim();
        if line.starts_with("#[") {
            if line.starts_with("#[repr(") {
                indices.push(prev);
            }
            i = prev;
        } else {
            break;
        }
    }
    indices.reverse();
    indices
}

fn brace_delta(line: &str) -> isize {
    line.chars().filter(|c| *c == '{').count() as isize
        - line.chars().filter(|c| *c == '}').count() as isize
}

fn parse_generated_field(line: &str) -> Option<GeneratedField> {
    let line = line.trim();
    let rest = line.strip_prefix("pub ")?;
    let (_name, ty) = rest.split_once(':')?;
    Some(GeneratedField {
        ty: ty.trim().trim_end_matches(',').to_owned(),
    })
}

fn m68k_pack_types(items: &[GeneratedItem], wide_aliases: &HashSet<String>) -> HashSet<String> {
    let mut pack_types = HashSet::new();

    for item in items {
        if item.has_aligned_repr {
            continue;
        }

        let has_wide_field = item
            .fields
            .iter()
            .any(|field| type_is_wide_scalar(&field.ty, wide_aliases));
        if !has_wide_field {
            continue;
        }

        pack_types.insert(item.name.clone());
    }

    pack_types
}

fn type_is_wide_scalar(ty: &str, wide_aliases: &HashSet<String>) -> bool {
    let ty = ty.trim();
    if ty.starts_with('*') {
        return false;
    }
    if let Some(rest) = ty.strip_prefix('[') {
        if let Some((element, _len)) = rest.split_once(';') {
            return type_is_wide_scalar(element.trim(), wide_aliases);
        }
    }
    if let Some(inner) = generic_argument(ty, "__IncompleteArrayField") {
        return type_is_wide_scalar(inner, wide_aliases);
    }
    if ty.starts_with("__BindgenBitfieldUnit")
        || ty.starts_with("__BindgenUnionField")
        || ty.starts_with("::core::marker::PhantomData")
    {
        return false;
    }

    wide_aliases.contains(ty)
}

fn generic_argument<'a>(ty: &'a str, wrapper: &str) -> Option<&'a str> {
    ty.strip_prefix(wrapper)?
        .trim_start()
        .strip_prefix('<')?
        .trim_end()
        .strip_suffix('>')
        .map(str::trim)
}

fn apply_m68k_packed_repr(
    bindings: &str,
    items: &[GeneratedItem],
    pack_types: &HashSet<String>,
) -> String {
    let mut lines = bindings.lines().map(str::to_owned).collect::<Vec<_>>();
    let mut drop_indices = HashSet::new();

    for item in items {
        if !pack_types.contains(&item.name) {
            continue;
        }
        if item
            .repr_indices
            .iter()
            .any(|idx| lines[*idx].contains("align"))
        {
            panic!(
                "m68k layout for {} needs packed(2), but bindgen emitted an aligned repr",
                item.name
            );
        }
        let Some(c_repr_idx) = item
            .repr_indices
            .iter()
            .copied()
            .find(|idx| lines[*idx].contains("repr(C"))
        else {
            panic!(
                "m68k layout for {} needs packed(2) without repr(C)",
                item.name
            );
        };
        if lines[c_repr_idx].contains("packed") {
            continue;
        }

        let indent_len = lines[c_repr_idx].len() - lines[c_repr_idx].trim_start().len();
        let indent = " ".repeat(indent_len);
        lines[c_repr_idx] = format!("{indent}#[repr(C, packed(2))]");
        if item
            .fields
            .iter()
            .any(|field| generic_argument(&field.ty, "__IncompleteArrayField").is_some())
        {
            remove_debug_derives(&mut lines, &mut drop_indices, item.item_index);
        }
    }

    let mut output = String::new();
    for (line_index, line) in lines.iter().enumerate() {
        if !drop_indices.contains(&line_index) {
            output.push_str(line);
            output.push('\n');
        }
    }
    output
}

fn remove_debug_derives(
    lines: &mut [String],
    drop_indices: &mut HashSet<usize>,
    item_index: usize,
) {
    let mut i = item_index;
    while let Some(prev) = i.checked_sub(1) {
        let line = lines[prev].trim();
        if !line.starts_with("#[") {
            break;
        }
        if line.starts_with("#[derive(") {
            if let Some(line) = remove_debug_from_derive_line(&lines[prev]) {
                lines[prev] = line;
            } else {
                drop_indices.insert(prev);
            }
        }
        i = prev;
    }
}

fn remove_debug_from_derive_line(line: &str) -> Option<String> {
    let Some((prefix, rest)) = line.split_once("#[derive(") else {
        return Some(line.to_owned());
    };
    let Some(inner) = rest.strip_suffix(")]") else {
        return Some(line.to_owned());
    };

    let derives = inner
        .split(',')
        .map(str::trim)
        .filter(|derive| *derive != "Debug")
        .collect::<Vec<_>>();
    if derives.is_empty() {
        None
    } else {
        Some(format!("{prefix}#[derive({})]", derives.join(", ")))
    }
}

/// Extract all `pub const` names from a generated .rs file.
fn extract_const_names(path: &str) -> HashSet<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter_map(|line| {
            line.strip_prefix("pub const ")
                .and_then(|rest| rest.split(':').next())
                .map(|name| name.trim().to_owned())
        })
        .collect()
}

/// Filters ioctl.rs using the two-pass set difference approach.
///
/// Constants present only in the with-fallback output came from function-like
/// macros (_IO*, v4l2_fourcc, etc.). For those: keep if not a known non-ioctl
/// and value >= 256. For object-like constants: keep if name matches a known
/// plain ioctl prefix.
fn filter_ioctl_two_pass(with_fb_path: &str, without_fb_path: &str) {
    let object_like_names = extract_const_names(without_fb_path);
    let content = fs::read_to_string(with_fb_path).unwrap();
    let mut output = String::new();

    for line in content.lines() {
        if line.starts_with("pub const ") {
            let rest = line.strip_prefix("pub const ").unwrap();
            let name = rest.split(':').next().unwrap_or("").trim();
            let is_fallback_only = !object_like_names.contains(name);

            let keep = if is_fallback_only {
                is_fallback_ioctl(name, rest)
            } else {
                is_plain_ioctl(name)
            };

            if keep {
                output.push_str(line);
                output.push('\n');
            }
        } else if line.starts_with("/* automatically generated")
            || line.starts_with("#![allow")
            || line.is_empty()
        {
            output.push_str(line);
            output.push('\n');
        }
    }

    fs::write(with_fb_path, output).unwrap();
}

/// Checks whether a function-like macro result is an ioctl constant.
fn is_fallback_ioctl(name: &str, rest: &str) -> bool {
    if FALLBACK_IOCTL_ALLOW.contains(&name) {
        return true;
    }
    if NON_IOCTL_PREFIX.iter().any(|p| name.starts_with(p)) {
        return false;
    }
    if NON_IOCTL_EXACT.contains(&name) {
        return false;
    }

    let value_str = rest
        .rsplit('=')
        .next()
        .unwrap_or("")
        .trim()
        .trim_end_matches(';')
        .trim();

    if let Ok(v) = value_str.parse::<u64>() {
        if v > u32::MAX as u64 {
            return false;
        }
        if v <= u8::MAX as u64 {
            return false;
        }
        return true;
    }

    false
}

fn is_plain_ioctl(name: &str) -> bool {
    PLAIN_IOCTL_INCLUDE.iter().any(|p| name.starts_with(p))
        && !PLAIN_IOCTL_EXCLUDE.iter().any(|p| name.starts_with(p))
}

fn compute_clang_target(rust_arch: &str) -> String {
    if rust_arch == "x86" {
        "i686-unknown-linux".to_string()
    } else if rust_arch == "x32" {
        "x86_64-unknown-linux-gnux32".to_string()
    } else if rust_arch == "mips32r6" {
        "mipsisa32r6-unknown-linux-gnu".to_string()
    } else if rust_arch == "mips64r6" {
        "mipsisa64r6-unknown-linux-gnuabi64".to_string()
    } else {
        format!("{}-unknown-linux", rust_arch)
    }
}
