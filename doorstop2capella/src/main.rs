use capella_reqif::req_if::{Object, ReqIf, SpecHierarchy, SpecObjectRequirement, Specification};
use chrono::{DateTime, Local, SecondsFormat};
use clap::Parser;
use doorstop_rs::doorstop::{document::Document, document_tree::DocumentTree};

fn complete(document: &Document, reqif: &mut ReqIf, specification: &mut Specification) {
    let local: DateTime<Local> = Local::now();
    let now = local.to_rfc3339_opts(SecondsFormat::Millis, false);
    let default_text_or_header = "".to_string();

    let root_children = &mut specification.children;

    for (_, each_item) in document.items.iter() {
        //add specs
        reqif.add_requirement(SpecObjectRequirement::new(
            each_item.id.as_ref().unwrap().to_string(),
            now.clone(),
            each_item
                .header
                .as_ref()
                .unwrap_or_else(|| &default_text_or_header)
                .to_owned(),
            each_item
                .text
                .as_ref()
                .unwrap_or_else(|| &default_text_or_header)
                .to_owned(),
            &reqif.core_content.req_if_content.spec_types,
        ));

        //Create the hierarchy
        let spec_hierarchy = SpecHierarchy {
            identifier: each_item.level.as_ref().unwrap().to_string(),
            last_change: now.clone(),
            object: Object {
                object_ref: each_item.id.as_ref().unwrap().to_string(),
            },
        };
        root_children.spec_hierarchy.push(spec_hierarchy);
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    ///YAML doorstop document root path
    document_path: String,
    ///ID for the specification in the reqif output file
    spec_id: String,
    ///Name for the specification in the reqif output file
    spec_name: String,
    ///Output file name
    #[clap(default_value = "out.reqif")]
    output_file: Option<String>,
}

fn main() {
    let cli = Args::parse();

    let document_tree = DocumentTree::load(&cli.document_path).unwrap();
    let mut reqif = ReqIf::new();
    let local: DateTime<Local> = Local::now();
    let now = local.to_rfc3339_opts(SecondsFormat::Millis, false);
    let mut specification = reqif.build_module_specification(cli.spec_id, now, cli.spec_name);

    complete(
        &document_tree.borrow().document,
        &mut reqif,
        &mut specification,
    );

    reqif.add_specification(specification);
    reqif.write_to(cli.output_file.as_ref().unwrap()).unwrap();
}
