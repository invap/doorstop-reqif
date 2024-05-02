// doorstop-reqif: Application that converts doorstop requirement into reqif format for capella mbse.
// Copyright (C) <2024>  INVAP S.E.
//
// This file is part of doorstop-reqif.
//
// doorstop-reqif is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use chrono::{DateTime, Local, SecondsFormat};
use clap::Parser;
use doorstop_rs::doorstop::{document::Document, document_tree::DocumentTree};
use reqif_rs::req_if::{Object, ReqIf, SpecHierarchy, SpecObjectRequirement, Specification};
use std::rc::Rc;

fn complete(document: &Document, reqif: &mut ReqIf, specification: &mut Specification) {
    let local: DateTime<Local> = Local::now();
    let now = local.to_rfc3339_opts(SecondsFormat::Millis, false);
    let default_text_or_header = "".to_string();

    let root_children = &mut specification.children;

    for (_, each_item) in document.items_sorted_by_level.iter() {
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
        let spec_hierarchy = SpecHierarchy::new(
            each_item.get_level().to_string(),
            now.clone(),
            Object::new(each_item.id.as_ref().unwrap().to_string()),
        );
        root_children
            .add_spec_hierarchy(spec_hierarchy, each_item.get_depth())
            .expect(format!("Error adding item {:#?}", each_item).as_str());
    }
}

fn create_specification(document: &Document, reqif: &mut ReqIf, local: DateTime<Local>) {
    let document_prefix = document.config.settings.prefix.as_str();

    let now = local.to_rfc3339_opts(SecondsFormat::Millis, false);
    let mut specification = reqif.build_module_specification(
        document_prefix.to_string(),
        now,
        format!("{} Specification", document_prefix),
    );

    complete(document, reqif, &mut specification);
    reqif.add_specification(specification);
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    ///YAML doorstop document root path
    document_path: String,
    ///Document prefix to export, if none all documents in tree are exported.
    doc_prefix: Option<String>,
    ///Output file name
    #[clap(default_value = "out.reqif")]
    output_file: Option<String>,
}

fn main() {
    let cli = Args::parse();

    let local: DateTime<Local> = Local::now();
    let document_tree = DocumentTree::load(&cli.document_path).unwrap();

    let identifier = "123456789A".to_string();
    let repository_id = "123456789A".to_string();
    let req_if_tool_id = "Doorstop".to_string();
    let source_tool_id = "Doorstop".to_string();
    let title = format!("Generated req if");

    let mut reqif = ReqIf::new(
        identifier,
        local,
        repository_id,
        req_if_tool_id,
        source_tool_id,
        title,
    );

    // let document = Rc::clone(&document_tree.as_ref().borrow().document);
    // create_specification(&document, &mut reqif, local);
    match cli.doc_prefix {
        None => {
            for (_, each_document_tree) in document_tree.borrow().prefix_index.borrow().iter() {
                create_specification(&each_document_tree.borrow().document, &mut reqif, local);
            }
        }
        Some(prefix) => {
            let document = Rc::clone(
                &document_tree
                    .borrow()
                    .prefix_index
                    .borrow()
                    .get(&prefix)
                    .expect(format!("Prefix {} not found in document tree", prefix).as_str())
                    .borrow()
                    .document,
            );
            create_specification(&document, &mut reqif, local);
        }
    }

    reqif.write_to(cli.output_file.as_ref().unwrap()).unwrap();
    println!("Export complete {:?} ", cli.output_file.unwrap());
}
