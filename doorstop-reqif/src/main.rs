// doorstop2capella: Application that converts doorstop requirement into reqif format for capella mbse.
// Copyright (C) <2024>  INVAP S.E.
//
// This file is part of doorstop2capella.
//
// doorstop2capella is free software: you can redistribute it and/or modify
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

    let local: DateTime<Local> = Local::now();
    let now = local.to_rfc3339_opts(SecondsFormat::Millis, false);
    let document_tree = DocumentTree::load(&cli.document_path).unwrap();

    let identifier = "123456789A".to_string();
    let repository_id = "123456789A".to_string();
    let req_if_tool_id = "Doorstop".to_string();
    let source_tool_id = "Doorstop".to_string();
    let title = format!("{}-({})", cli.spec_name, cli.spec_id);

    let mut reqif = ReqIf::new(
        identifier,
        local,
        repository_id,
        req_if_tool_id,
        source_tool_id,
        title,
    );

    let mut specification = reqif.build_module_specification(cli.spec_id, now, cli.spec_name);

    complete(
        &document_tree.borrow().document,
        &mut reqif,
        &mut specification,
    );

    reqif.add_specification(specification);
    reqif.write_to(cli.output_file.as_ref().unwrap()).unwrap();
}
