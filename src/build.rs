extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res: winres::WindowsResource = winres::WindowsResource::new();
        res.set_icon(
            "/home/linuxu/Documentos/Programming Workspace/Nyx Programming Language/nyx/assets/Nyx Programming Language.png",
        );

        res.set_version_info(winres::VersionInfo::FILEVERSION, 1);
        res.compile().unwrap();
    }
}
