mod common;
use common::*;
use sruja_extract::terraform::TerraformExtractor;

#[test]
fn terraform_extractor_name() {
    assert_eq!(TerraformExtractor::new().name(), "terraform");
}

#[test]
fn terraform_extractor_detects_resources() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("main.tf");
    fs::write(
        &file_path,
        r#"
resource "aws_ecs_service" "payment" {
  name = "payment-svc"
}

resource "aws_rds_instance" "orders_db" {
  engine = "postgres"
}

module "vpc" {
  source = "./modules/vpc"
}
"#,
    )
    .unwrap();

    let results = check(&TerraformExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].binding.kind, SourceKind::Terraform);
}

#[test]
fn terraform_extractor_ignores_non_tf() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("main.rs");
    fs::write(&file_path, "fn main() {}").unwrap();

    let results = check(&TerraformExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn terraform_extractor_resources_higher_confidence() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("main.tf");
    fs::write(
        &file_path,
        "resource \"aws_lambda_function\" \"handler\" {\n}\n\nmodule \"utils\" {\n  source = \"./m\"\n}\n",
    )
    .unwrap();

    let results = check(&TerraformExtractor::new(), &file_path, tmp.path());
    let resource = results
        .iter()
        .find(|r| {
            r.binding
                .description
                .as_deref()
                .unwrap()
                .contains("resource.")
        })
        .unwrap();
    let module = results
        .iter()
        .find(|r| r.binding.description.as_deref().unwrap().contains("module"))
        .unwrap();
    assert!(resource.confidence > module.confidence);
}

#[test]
fn terraform_extractor_detects_data_blocks() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("data.tf");
    fs::write(
        &file_path,
        "data \"aws_vpc\" \"main\" {\n  default = true\n}\n",
    )
    .unwrap();

    let results = check(&TerraformExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    let desc = results[0].binding.description.as_deref().unwrap();
    assert!(desc.contains("data.aws_vpc"));
    assert!(
        results[0].suggested_element.is_none(),
        "data blocks should not suggest elements"
    );
}
