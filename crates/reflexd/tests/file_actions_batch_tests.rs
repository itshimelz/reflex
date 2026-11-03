use reflexd::file_actions::*;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_multi_selection_duplicate_and_clean() {
    let tmp = tempdir().unwrap();
    let file1 = tmp.path().join("My File (1).txt");
    let file2 = tmp.path().join("Another Document!.pdf");
    fs::write(&file1, "hello 1").unwrap();
    fs::write(&file2, "hello 2").unwrap();

    let paths = vec![file1.clone(), file2.clone()];

    // 1. Duplicate both
    let dup_res = duplicate_selected(&paths).await;
    assert!(dup_res.is_ok());
    assert_eq!(dup_res.unwrap(), "Duplicated 2 item(s)");
    assert!(tmp.path().join("My File (1)_copy.txt").exists());
    assert!(tmp.path().join("Another Document!_copy.pdf").exists());

    // 2. Clean filenames
    let clean_res = clean_filename(&paths).await;
    assert!(clean_res.is_ok());
    assert_eq!(clean_res.unwrap(), "Cleaned 2 filename(s)");
    assert!(tmp.path().join("my-file-1.txt").exists());
    assert!(tmp.path().join("another-document.pdf").exists());
}

#[tokio::test]
async fn test_multi_selection_timestamp() {
    let tmp = tempdir().unwrap();
    let file1 = tmp.path().join("report.docx");
    let file2 = tmp.path().join("data.csv");
    fs::write(&file1, "docx").unwrap();
    fs::write(&file2, "csv").unwrap();

    let paths = vec![file1, file2];
    let res = timestamp_selected(&paths).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "Timestamped 2 file(s)");

    let entries: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(entries.len(), 2);
    for name in entries {
        assert!(name.contains("_report.docx") || name.contains("_data.csv"));
    }
}

#[tokio::test]
async fn test_multi_selection_file_info() {
    let tmp = tempdir().unwrap();
    let file1 = tmp.path().join("a.txt");
    let file2 = tmp.path().join("b.txt");
    fs::write(&file1, "12345").unwrap(); // 5 bytes
    fs::write(&file2, "1234567890").unwrap(); // 10 bytes

    let paths = vec![file1, file2];
    let res = file_info(&paths).await;
    assert!(res.is_ok());
    let info = res.unwrap();
    assert!(info.contains("2 items: 15 B total (2 files)"));
}

#[tokio::test]
async fn test_multi_selection_rename() {
    let tmp = tempdir().unwrap();
    let file1 = tmp.path().join("img1.png");
    let file2 = tmp.path().join("img2.png");
    fs::write(&file1, "png1").unwrap();
    fs::write(&file2, "png2").unwrap();

    let paths = vec![file1, file2];
    let res = rename_selected(&paths, "photo").await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "Renamed 2 items to 'photo_N'");
    assert!(tmp.path().join("photo_1.png").exists());
    assert!(tmp.path().join("photo_2.png").exists());
}
