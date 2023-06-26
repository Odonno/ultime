use anyhow::Result;
use assert_fs::fixture::PathChild;

use crate::helpers::*;

#[test]
fn fails_to_generate_endpoint_if_both_from_query_and_mutation() -> Result<()> {
    let temp_dir = assert_fs::TempDir::new()?;

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&temp_dir)
            .arg("new")
            .arg("my-blog")
            .arg("--template")
            .arg("blog");

        cmd.assert().success();
    }

    let project_dir = temp_dir.child("my-blog");

    let mut cmd = create_cmd()?;
    cmd.current_dir(&project_dir)
        .arg("generate")
        .arg("endpoint")
        .arg("test")
        .arg("--from-query")
        .arg("posts")
        .arg("--from-mutation")
        .arg("comment");

    cmd.assert()
        .failure()
        .stderr("error: the argument \'--from-query <FROM_QUERY>\' cannot be used with \'--from-mutation <FROM_MUTATION>\'

Usage: ultime generate endpoint --from-query <FROM_QUERY> <NAME>

For more information, try \'--help\'.\n");

    temp_dir.close()?;

    Ok(())
}
