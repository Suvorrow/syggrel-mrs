// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

// Import COOL types (stub pr real, depending on feature flag)
use file_picker::{pick_genomic_input, pick_serialization_output};
use parser::hic_parser::{HiCConfig, NormalizationMethod, extract_contact_matrix as extract_hic};
use parser::parser_utils::save_matrix_to_path;
use parser::{
    CoolConfig, extract_contact_matrix as extract_cool, format_matrix_stats, format_output_intent,
    format_save_confirmation,
};

#[derive(Debug)]
pub struct ParseError(pub &'static str);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ParseError {}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ParserFormat {
    Hic,
    Cool,
}

impl ParserFormat {
    fn as_str(&self) -> &'static str {
        match self {
            ParserFormat::Hic => "HIC",
            ParserFormat::Cool => "COOL",
        }
    }
}

impl FromStr for ParserFormat {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "COOL" => Ok(ParserFormat::Cool),
            "HIC" => Ok(ParserFormat::Hic),
            _ => Err(ParseError("Invalid format. Expected 'COOL' or 'HIC'")),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ParserToolViewProps {
    pub close_tool: EventHandler<()>,
}

// --------------------------------------------------------------------------------------------
// Main component
// --------------------------------------------------------------------------------------------
#[component]
pub fn ParserToolView(props: ParserToolViewProps) -> Element {
    let close_tool = props.close_tool;

    let mut file_format = use_signal(|| ParserFormat::Cool);

    // Common input state for UI elements
    let mut input_path = use_signal(|| String::new());
    let mut output_path = use_signal(|| String::new());
    let mut resolution = use_signal(|| "1000000".to_owned());
    let mut normalization_type = use_signal(|| "NONE".to_owned());

    // Matrix viewer/editor state
    let mut extracted_matrix = use_signal(|| Option::<Arc<(usize, usize, Vec<f64>)>>::None);
    let mut display_limit = use_signal(|| 50);

    // HiC-specific state
    let mut chromosome = use_signal(|| "1".to_owned());
    let mut start_bin = use_signal(|| String::new());
    let mut end_bin = use_signal(|| String::new());

    // COOL-specific state
    let mut cool_group = use_signal(|| "resolutions".to_owned());

    // UI state
    let mut is_parsing = use_signal(|| false);
    let mut progress = use_signal(|| 0.0);
    let mut status_message = use_signal(|| String::new());

    let mut log_messages = use_signal(|| {
        vec![
            "🚀 Parser initialized".to_owned(),
            "💡 Tip: Load anyfile.cool with resolution 1000000 to test".to_owned(),
        ]
    });

    let progress_text = use_memo(move || {
        let percent = progress() * 100.0;
        format!("Progress: {:.2}%", percent)
    });

    // Save result handler
    let on_save_result = {
        let matrix_signal = extracted_matrix;
        let mut status = status_message;
        let mut logs = log_messages;

        move |_: MouseEvent| {
            let matrix_data = matrix_signal.read().clone();

            spawn(async move {
                match matrix_data {
                    Some(arc_data) => {
                        let (rows, cols, flat_data) = Arc::unwrap_or_clone(arc_data);
                        status.set("💾 Preparing save...".into());

                        if let Some(path) = pick_serialization_output().await {
                            status.set("💾 Writing to disk...".into());
                            match save_matrix_to_path(rows, cols, &flat_data, &path) {
                                Ok(_) => {
                                    let msg = format!("✅ Saved to: {}", path.display());
                                    status.set(msg.clone());
                                    logs.write().push(msg);
                                }
                                Err(e) => {
                                    let msg = format!("❌ Save failed: {e}");
                                    status.set(msg.clone());
                                    logs.write().push(msg);
                                }
                            }
                        } else {
                            status.set("⚠️ Save cancelled by user".into());
                        }
                    }
                    None => {
                        let msg = "⚠️ No matrix available to save".to_owned();
                        status.set(msg.clone());
                        logs.write().push(msg);
                    }
                }
            });
        }
    };

    //-----------------------------------------------------------------------------------------
    // Start parsing handler
    //-----------------------------------------------------------------------------------------
    let start_parsing = move |_| {
        if input_path.read().is_empty() {
            let msg: String = "Error: Please specify the input file path.".into();
            status_message.set(msg.clone());
            log_messages.write().push(msg);
            return;
        }

        is_parsing.set(true);
        status_message.set("Initializing parse...".into());
        progress.set(0.0);

        {
            let mut logs = log_messages.write();
            logs.clear();
            logs.push("🔄 Starting parse process...".into());
        }

        // Capture format and shared values.
        let format = file_format();
        let input_path_val = input_path.read().clone();
        let output_path_val = output_path.read().clone();

        // Clone signals for async block.
        let (mut logs_sig, mut status_sig, mut parsing_sig) =
            (log_messages, status_message, is_parsing);

        //-------------------------------------------------------------------------------------
        // Async parsing work
        //-------------------------------------------------------------------------------------
        spawn(async move {
            match format {
                ParserFormat::Hic => {
                    // Clone only HIC-specific values here (avoid unused allocs)
                    let chromosome_val = chromosome.read().clone();
                    let resolution_val = resolution.read().clone();
                    let norm_type_val = normalization_type.clone();
                    let start_bin_val = start_bin.read().clone();
                    let end_bin_val = end_bin.read().clone();

                    // Parse configuration values.
                    let start_bin_parsed: Option<usize> = start_bin_val.parse().ok();
                    let end_bin_parsed: Option<usize> = end_bin_val.parse().ok();

                    let config = match resolution_val.parse::<u32>() {
                        Ok(res) => HiCConfig {
                            normalization: match norm_type_val.read().as_str() {
                                "VC" => NormalizationMethod::VC,
                                "VCSQRT" => NormalizationMethod::VCSQRT,
                                "KR" => NormalizationMethod::KR,
                                _ => NormalizationMethod::NONE,
                            },
                            resolution: res,
                            chromosome: chromosome_val,
                            start_bin: start_bin_parsed,
                            end_bin: end_bin_parsed,
                        },
                        Err(e) => {
                            let err_msg = format!("Error parsing resolution: {e}");
                            logs_sig.write().push(err_msg.clone());
                            status_sig.set(err_msg);
                            parsing_sig.set(false);
                            return;
                        }
                    };

                    let output_path_opt = if !output_path_val.is_empty() {
                        Some(Path::new(&output_path_val))
                    } else {
                        None
                    };

                    {
                        let mut w = logs_sig.write();
                        w.push(format_output_intent(output_path_opt.as_deref()));
                        w.push("Calling hictk via hic_parser...".into());
                    }
                    status_sig.set("Calling hictk...".into());

                    match extract_hic(&input_path_val, &config, output_path_opt) {
                        Ok(matrix) => {
                            parsing_sig.set(false);
                            status_sig.set("Parsing completed successfully!".into());

                            {
                                let mut w = logs_sig.write();
                                w.push("Parse process finished. Matrix extracted.".into());
                                w.push(format!(
                                    "Extracted matrix dimensions: {:?}",
                                    matrix.shape()
                                ));
                                w.push(format_save_confirmation(output_path_opt.as_deref()));
                            }

                            extracted_matrix.set(Some(Arc::new((
                                matrix.nrows(),
                                matrix.ncols(),
                                matrix.iter().copied().collect(),
                            ))));
                        }
                        Err(e) => {
                            parsing_sig.set(false);
                            let err_msg = format!("❌ HiC parsing failed: {e}");
                            status_sig.set(err_msg.clone());
                            logs_sig.write().push(err_msg);
                        }
                    }
                }

                //-----------------------------------------------------------------------------
                // COOL parsing branch
                //-----------------------------------------------------------------------------
                ParserFormat::Cool => {
                    // Clone only COOL-specific values here (avoid unused allocs)
                    let chromosome_val = chromosome.read().clone();
                    let resolution_val = resolution.read().clone();
                    let norm_type_val = normalization_type.read().clone();
                    let start_bin_val = start_bin.read().clone();
                    let end_bin_val = end_bin.read().clone();
                    let cool_group_val = cool_group.read().clone();

                    {
                        let mut w = logs_sig.write();
                        w.push("📦 Parsing COOL file...".into());
                    }
                    status_sig.set("Processing .cool format...".into());

                    let resolution_parsed = match resolution_val.parse::<u32>() {
                        Ok(res) => res,
                        Err(e) => {
                            let err_msg = format!("Error parsing resolution: {e}");
                            logs_sig.write().push(err_msg.clone());
                            status_sig.set(err_msg);
                            parsing_sig.set(false);
                            return;
                        }
                    };

                    let cool_config = CoolConfig {
                        resolution: resolution_parsed,
                        chromosome: if chromosome_val.is_empty() {
                            None
                        } else {
                            Some(chromosome_val)
                        },
                        bin_range: if !start_bin_val.is_empty() && !end_bin_val.is_empty() {
                            match (start_bin_val.parse::<usize>(), end_bin_val.parse::<usize>()) {
                                (Ok(s), Ok(e)) if s < e => Some((s, e)),
                                _ => None,
                            }
                        } else {
                            None
                        },
                        normalization: if norm_type_val == "NONE" || norm_type_val.is_empty() {
                            None
                        } else {
                            Some(norm_type_val)
                        },
                        group_path: if cool_group_val.is_empty() {
                            "resolutions".into()
                        } else {
                            cool_group_val
                        },
                    };

                    logs_sig.write().push(format!(
                        "Config: res={}, group='{}', chr={:?}",
                        cool_config.resolution, cool_config.group_path, cool_config.chromosome
                    ));

                    let output_path_opt = if !output_path_val.is_empty() {
                        Some(Path::new(&output_path_val))
                    } else {
                        None
                    };

                    {
                        let mut w = logs_sig.write();
                        w.push(format_output_intent(output_path_opt.as_deref()));
                        w.push("🚀 Calling extract_cool()...".into());
                    }

                    match extract_cool(&input_path_val, &cool_config, output_path_opt) {
                        Ok(matrix) => {
                            parsing_sig.set(false);
                            status_sig.set("✅ COOL parsing completed!".into());

                            {
                                let mut w = logs_sig.write();
                                w.push("📊 Parse process finished. Matrix extracted.".into());
                                for msg in format_matrix_stats(&matrix) {
                                    w.push(msg);
                                }
                                w.push(format_save_confirmation(output_path_opt.as_deref()));
                            }

                            // Allocates dense Vec<f64> for UI serialization.
                            // Arc prevents redundant copies when signal is cloned.
                            extracted_matrix.set(Some(Arc::new((
                                matrix.nrows(),
                                matrix.ncols(),
                                matrix.iter().copied().collect(),
                            ))));
                        }
                        Err(e) => {
                            parsing_sig.set(false);
                            let err_msg = format!("❌ COOL parsing failed: {e}");
                            status_sig.set(err_msg.clone());
                            {
                                let mut w = logs_sig.write();
                                w.push(err_msg);
                                w.push(format!("🔍 Error type: {:?}", e));
                            }
                        }
                    }
                }
            }
        });
    };

    // Browse button for INPUT file.
    let on_browse_input = move |_| {
        spawn(async move {
            if let Some(path) = pick_genomic_input().await {
                input_path.set(path.to_string_lossy().into());
            }
        });
    };

    let on_browse_save = move |_: MouseEvent| {
        spawn(async move {
            if let Some(path) = pick_serialization_output().await {
                output_path.set(path.to_string_lossy().into());
            }
        });
    };

    // ----------------------------------------------------------------------------------------
    // Render format-specific parameters (dynamic UI)
    // ----------------------------------------------------------------------------------------
    let render_format_params = move || -> Element {
        match file_format() {
            ParserFormat::Hic => rsx! {
                div { class: "control-group",
                    label { "Chromosome:" }
                    input {
                        r#type: "text",
                        value: "{chromosome}",
                        disabled: is_parsing(),
                        onchange: move |evt| chromosome.set(evt.value()),
                    }
                }
                div { class: "control-group",
                    label { "Resolution (bins):" }
                    input {
                        r#type: "text",
                        value: "{resolution}",
                        disabled: is_parsing(),
                        onchange: move |evt| resolution.set(evt.value()),
                    }
                }
                div { class: "control-group",
                    label { "Start Bin (optional):" }
                    input {
                        r#type: "text",
                        value: "{start_bin}",
                        disabled: is_parsing(),
                        onchange: move |evt| start_bin.set(evt.value()),
                    }
                }
                div { class: "control-group",
                    label { "End Bin (optional):" }
                    input {
                        r#type: "text",
                        value: "{end_bin}",
                        disabled: is_parsing(),
                        onchange: move |evt| end_bin.set(evt.value()),
                    }
                }
                div { class: "control-group",
                    label { "Normalization:" }
                    select {
                        value: "{normalization_type}",
                        disabled: is_parsing(),
                        onchange: move |evt| normalization_type.set(evt.value()),
                        option { value: "NONE", "None" }
                        option { value: "VC", "Vanilla Coverage (VC)" }
                        option { value: "VCSQRT", "Vanilla Coverage Sqrt (VCSQRT)" }
                        option { value: "KR", "Knight-Ruiz (KR)" }
                    }
                }
            },
            ParserFormat::Cool => rsx! {
                div { class: "control-group",
                    label { "Group:" }
                    input {
                        r#type: "text",
                        value: "{cool_group}",
                        placeholder: "resolutions",
                        disabled: is_parsing(),
                        onchange: move |evt| cool_group.set(evt.value()),
                    }
                }
                div { class: "control-group",
                    label { "Resolution:" }
                    input {
                        r#type: "text",
                        value: "{resolution}",
                        disabled: is_parsing(),
                        onchange: move |evt| resolution.set(evt.value()),
                    }
                }
                div { class: "control-group",
                    label { "Normalization (optional):" }
                    select {
                        value: "{normalization_type}",
                        disabled: is_parsing(),
                        oninput: move |evt| normalization_type.set(evt.value()),
                        option { value: "NONE", "None" }
                        option { value: "KR", "Knight-Ruiz (KR)" }
                        option { value: "VC", "Vanilla Coverage (VC)" }
                    }
                }
            },
        }
    };

    //-----------------------------------------------------------------------------------------
    // UI Rendering (synchronous, outside spawn)
    //-----------------------------------------------------------------------------------------
    let logs_snapshot = log_messages.read().clone();
    let input_placeholder = match file_format() {
        ParserFormat::Hic => "/home/path/to/myfile.hic",
        ParserFormat::Cool => "/home/path/to/myfile.cool",
    };
    let progress_width = format!("width: {}%;", progress() * 100.0);
    let matrix_opt = extracted_matrix.read().clone();

    //  Pre-extract matrix data BEFORE rsx! (avoids RSX parser limitations)
    let has_matrix = matrix_opt.is_some();
    let r_rows = matrix_opt.as_ref().map_or(0, |d| d.0);
    let r_cols = matrix_opt.as_ref().map_or(0, |d| d.1);
    let limit = display_limit();
    let empty_vec = Vec::<f64>::new();
    let mat: &Vec<f64> = matrix_opt.as_ref().map_or(&empty_vec, |d| &d.2);

    //  Build matrix preview content as a separate variable
    let matrix_preview_content = if has_matrix {
        rsx! {
            div { class: "matrix-editor-panel",
                h3 { "📊 Extracted Matrix Preview" }
                div { class: "editor-toolbar",
                    span { "Showing first {r_rows.min(limit)}x{r_cols.min(limit)} of {r_rows}x{r_cols}" }
                    input {
                        r#type: "number",
                        class: "limit-input",
                        value: limit.to_string(),
                        min: "5", max: "200",
                        oninput: move |evt| {
                            if let Ok(v) = evt.value().parse::<usize>() {
                                display_limit.set(v.min(200));
                            }
                        },
                    }
                    button {
                        class: "save-matrix-btn",
                        disabled: !has_matrix,
                        onclick: on_save_result,
                        "💾 Save Full Matrix..."
                    }
                }

                div { class: "matrix-grid-container",
                    for r in 0..r_rows.min(limit) {
                        div { class: "matrix-row",
                            for c in 0..r_cols.min(limit) {
                                div {
                                    key: "{r}-{c}",
                                    class: "matrix-cell",
                                    {format!("{:.4}", mat[r * r_cols + c])}
                                }
                            }
                        }
                    }
                    if r_rows > limit || r_cols > limit {
                        p {
                            class: "matrix-note",
                            "[WARN] Large matrix: UI shows a preview only. Use 'Save Full Matrix...' to export complete data."
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    };

    // Main RSX Block
    rsx! {
        div { class: "parser-overlay",
            div { class: "parser-header",
                h2 { "Universal Parser Tool" }
                button {
                    class: "close-parser-btn",
                    onclick: move |_| close_tool.call(()),
                    "✕"
                }
            }

            div { class: "parser-main-layout",
                div { class: "parser-sidebar",
                    h3 { "Parsing Parameters" }

                    div { class: "control-group",
                        label { "File Format:" }
                        select {
                            value: file_format().as_str(),
                            disabled: is_parsing(),
                            onchange: move |evt| {
                                if let Ok(fmt) = ParserFormat::from_str(&evt.value()) {
                                    file_format.set(fmt);
                                }
                            },
                            option { value: "HIC", "HiC (.hic)" }
                            option { value: "COOL", "Cooler (.cool)" }
                        }
                    }

                    div { class: "control-group",
                        label { "Input File:" }
                        div { class: "input-with-button",
                            input {
                                r#type: "text",
                                value: input_path(),
                                placeholder: input_placeholder,
                                disabled: is_parsing(),
                                oninput: move |evt| input_path.set(evt.value()),
                            }
                            button {
                                class: "browse-btn",
                                disabled: is_parsing(),
                                onclick: on_browse_input,
                                "Browse..."
                            }
                        }
                    }

                    div { class: "control-group",
                        label { "Save Output File:" }
                        div { class: "input-with-button",
                            input {
                                r#type: "text",
                                value: output_path(),
                                placeholder: "/home/save/to_path/myfile.csv",
                                disabled: is_parsing(),
                                oninput: move |evt| output_path.set(evt.value()),
                            }
                            button {
                                class: "browse-btn",
                                disabled: is_parsing(),
                                onclick: on_browse_save,
                                "📂 Save as..."
                            }
                        }
                    }

                    {render_format_params()}

                    button {
                        class: "parse-btn",
                        disabled: is_parsing() || input_path().is_empty(),
                        onclick: start_parsing,
                        "Start {file_format().as_str()} Parse"
                    }
                }

                div { class: "parser-results",
                    h3 { "Parsing Results & Progress" }
                    div { class: "status-message", "{status_message()}" }

                    div { class: "progress-container",
                        div { class: "progress-bar", style: progress_width }
                    }

                    p { "{progress_text()}" }

                    div { class: "parser-preview-area",
                        {matrix_preview_content}
                    }

                    div { class: "parser-logs",
                        h4 { "Log Output" }
                        div { class: "log-content",
                            for log in logs_snapshot {
                                p { "{log}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
