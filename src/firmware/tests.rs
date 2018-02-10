use super::*;
use mktemp::Temp;
use std::path::PathBuf;

pub fn create_hook(path: PathBuf, contents: &str, mode: u32) {
    use std::fs::File;
    use std::fs::create_dir_all;
    use std::fs::metadata;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    // ensure path exists
    create_dir_all(path.parent().unwrap()).unwrap();

    let mut file = File::create(&path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();

    let mut permissions = metadata(path).unwrap().permissions();
    permissions.set_mode(mode);
    file.set_permissions(permissions).unwrap();
}

pub fn product_uid_hook(path: &Path) -> PathBuf {
    path.join(PRODUCT_UID_HOOK)
}

pub fn version_hook(path: &Path) -> PathBuf {
    path.join(VERSION_HOOK)
}

pub fn hardware_hook(path: &Path) -> PathBuf {
    path.join(HARDWARE_HOOK)
}

pub fn device_identity_dir(path: &Path) -> PathBuf {
    path.join(DEVICE_IDENTITY_DIR).join("identity")
}

pub fn device_attributes_dir(path: &Path) -> PathBuf {
    path.join(DEVICE_ATTRIBUTES_DIR).join("attributes")
}

pub enum FakeDevice {
    NoUpdate,
    HasUpdate,
    ExtraPoll,
}

pub fn create_fake_metadata(device: FakeDevice) -> PathBuf {
    let tmpdir = Temp::new_dir().unwrap().to_path_buf();

    // create fake hooks to be used to validate the load
    create_hook(
        product_uid_hook(&tmpdir),
        "#!/bin/sh\necho 229ffd7e08721d716163fc81a2dbaf6c90d449f0a3b009b6a2defe8a0b0d7381",
        0o755,
    );
    create_hook(version_hook(&tmpdir), "#!/bin/sh\necho 1.1", 0o755);
    create_hook(hardware_hook(&tmpdir), "#!/bin/sh\necho board", 0o755);
    create_hook(
        device_identity_dir(&tmpdir),
        &format!(
            "#!/bin/sh\necho id1=value{}\necho id2=value2",
            match device {
                FakeDevice::NoUpdate => 1,
                FakeDevice::HasUpdate => 2,
                FakeDevice::ExtraPoll => 3,
            }
        ),
        0o755,
    );
    create_hook(
        device_attributes_dir(&tmpdir),
        "#!/bin/sh\necho attr1=attrvalue1\necho attr2=attrvalue2",
        0o755,
    );

    tmpdir
}

#[test]
fn run_multiple_hooks_in_a_dir() {
    let tmpdir = Temp::new_dir().unwrap();

    // create two scripts so we can test the parsing of output
    create_hook(
        tmpdir.to_path_buf().join("hook1"),
        "#!/bin/sh\necho key2=val2\necho key1=val1",
        0o755,
    );
    create_hook(
        tmpdir.to_path_buf().join("hook2"),
        "#!/bin/sh\necho key2=val4\necho key1=val3",
        0o755,
    );

    let fv = run_hooks_from_dir(&tmpdir.to_path_buf()).unwrap();

    assert_eq!(fv.keys().len(), 2);
    assert_eq!(fv.keys().collect::<Vec<_>>(), ["key1", "key2"]);
    assert_eq!(fv["key1"], ["val1", "val3"]);
    assert_eq!(fv["key2"], ["val2", "val4"]);
}

#[test]
fn check_load_metadata() {
    use std::fs::remove_file;

    {
        let metadata_dir = create_fake_metadata(FakeDevice::NoUpdate);
        // check error with a invalid product uid
        create_hook(
            product_uid_hook(&metadata_dir),
            "#!/bin/sh\necho 123",
            0o755,
        );
        let metadata = Metadata::new(&metadata_dir);
        assert!(metadata.is_err());
    }

    {
        // check error when lacks product uid
        let metadata_dir = create_fake_metadata(FakeDevice::NoUpdate);
        remove_file(product_uid_hook(&metadata_dir)).unwrap();
        let metadata = Metadata::new(&metadata_dir);
        assert!(metadata.is_err());
    }

    {
        // check error when lacks device identity
        let metadata_dir = create_fake_metadata(FakeDevice::NoUpdate);
        remove_file(device_identity_dir(&metadata_dir)).unwrap();
        let metadata = Metadata::new(&metadata_dir);
        assert!(metadata.is_err());
    }

    {
        // check if is still valid without device attributes
        let metadata_dir = create_fake_metadata(FakeDevice::NoUpdate);
        remove_file(device_attributes_dir(&metadata_dir)).unwrap();
        let metadata = Metadata::new(&metadata_dir).unwrap();
        assert_eq!(
            "229ffd7e08721d716163fc81a2dbaf6c90d449f0a3b009b6a2defe8a0b0d7381",
            metadata.product_uid
        );
        assert_eq!("1.1", metadata.version);
        assert_eq!("board", metadata.hardware);
        assert_eq!(2, metadata.device_identity.len());
        assert_eq!(0, metadata.device_attributes.len());
    }

    {
        // complete metadata
        let metadata_dir = create_fake_metadata(FakeDevice::NoUpdate);
        let metadata = Metadata::new(&metadata_dir).unwrap();
        assert_eq!(
            "229ffd7e08721d716163fc81a2dbaf6c90d449f0a3b009b6a2defe8a0b0d7381",
            metadata.product_uid
        );
        assert_eq!("1.1", metadata.version);
        assert_eq!("board", metadata.hardware);
        assert_eq!(2, metadata.device_identity.len());
        assert_eq!(2, metadata.device_attributes.len());
    }
}
