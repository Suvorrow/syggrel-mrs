#![allow(non_snake_case)]

use std::path::Path;
use dioxus::prelude::*;
use mrs_exploring_kit::parser::hic_parser::{
    HiCConfig,
    NormalizationMethod,
    extract_contact_matrix as extract_hic,
};
// Import COOL types (stub pr real, depending on feature flag)
use mrs_exploring_kit::parser::{
    CoolConfig, extract_contact_matrix as extract_cool,
    format_matrix_stats,
    format_output_intent,
    format_save_confirmation,
};
use mrs_exploring_kit::parser::parser_utils::save_matrix_to_path;
use file_picker::{pick_genomic_input, pick_serialization_output};

// Universal format enum
#[derive(Clone, Copy, PartialEq)]
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
    fn from_str(s: &str) -> Self {
        match s {
            "COOL" => ParserFormat::Cool,
            _ => ParserFormat::Hic,
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
    let mut input_path = use_signal(String::new);
    let mut output_path = use_signal(String::new);
    let mut resolution = use_signal(|| String::from("1000000"));
    let mut normalization_type = use_signal(|| String::from("NONE"));

    // Matrix viewer/editor state
    let mut extracted_matrix = use_signal(|| Option::<(usize, usize, Vec<f64>)>::None);
    let display_limit = use_signal(|| 50);

    // HiC-specific state
    let mut chromosome = use_signal(|| String::from("1"));
    let mut start_bin = use_signal(String::new);
    let mut end_bin = use_signal(String::new);

    // COOL-specific state
    let mut cool_group = use_signal(|| String::from("resolutions"));

    // UI state
    let mut is_parsing = use_signal(|| false);
    let mut progress = use_signal(|| 0.0);
    let mut status_message = use_signal(String::new);
    let mut log_messages = use_signal(|| vec![
        "🚀 Parser initialized".to_string(),
        "💡 Tip: Load anyfile.cool with resolution 1000000 to test".to_string(),
    ]);
    let progress_percent = progress() * 100.0;
    let progress_text = format!("Progress: {:.2}%", progress_percent);

    // Save result handler
    let on_save_result = move |_: MouseEvent| {
        let matrix_data = extracted_matrix.read().clone();
        spawn(async move {
            match matrix_data {
                // Matrix exists - proceed with save dialog
                Some((rows, cols, flat_data)) => {
                    if let Some(path) = file_picker::pick_serialization_output().await {
                        match save_matrix_to_path(rows, cols, &flat_data, &path) {
                            Ok(_) => println!("✅ Saved to: {}", path.display()),
                            Err(e) => eprintln!("❌ Save failed: {e}"),
                        }
                    }
                }
                // No matrix to save - log and exit gracefully
                None => { println!("⚠️ No matrix available to save") },
            }
        });
    };

    //-----------------------------------------------------------------------------------------
    // Start parsing handler
    //-----------------------------------------------------------------------------------------
    let start_parsing = move |_| {
        if input_path.read().is_empty() {
            let msg = "Error: Please specify the input file path.".to_string();
            status_message.set(msg.clone());
            log_messages.write().push(msg);
            return;
        }

        is_parsing.set(true);
        status_message.set("Initializing parse...".to_string());
        progress.set(0.0);
        log_messages.set(vec![String::from("🔄 Starting parse process... ")]);

        // Capture format and shared values.
        let format = file_format();
        let input_path_val = input_path.read().as_str().to_owned();
        let output_path_val = output_path.read().as_str().to_owned();

        // Clone signals for async block.
        let (mut log_messages_clone, mut status_message_clone, mut is_parsing_clone) =
            (log_messages, status_message, is_parsing);

        //-------------------------------------------------------------------------------------
        // Async parsing work
        //-------------------------------------------------------------------------------------
        spawn(async move {
            match format {
                ParserFormat::Hic => {
                    // Clone only HIC-specific values here (avoid unused allocs)
                    let chromosome_val = chromosome.read().as_str().to_owned();
                    let resolution_val = resolution.read().as_str().to_owned();
                    let norm_type_val = normalization_type.read().as_str().to_owned();
                    let start_bin_val = start_bin.read().as_str().to_owned();
                    let end_bin_val = end_bin.read().as_str().to_owned();

                    // Parse configuration values.
                    let res_parsed: Result<u32, _> = resolution_val.parse();
                    let start_bin_parsed: Option<usize> = start_bin_val.parse().ok();
                    let end_bin_parsed: Option<usize> = end_bin_val.parse().ok();

                    let config = match res_parsed {
                        Ok(res) => HiCConfig {
                            normalization: match norm_type_val.as_str() {
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
                            let err_msg = format!("Error parsing resolution: {}", e);
                            log_messages_clone.write().push(err_msg.clone());
                            status_message_clone.set(err_msg);
                            is_parsing_clone.set(false);
                            return;
                        }
                    };

                    let output_path_opt = if !output_path_val.is_empty() {
                        Some(std::path::Path::new(&output_path_val))
                    } else {
                        None
                    };

                    log_messages_clone.write().push(format_output_intent(output_path_opt.as_deref()));

                    log_messages_clone.write().push("Calling hictk via hic_parser...".to_string());
                    status_message_clone.set("Calling hictk...".to_string());

                    match extract_hic(&input_path_val, &config, output_path_opt) {
                        Ok(matrix) => {
                            is_parsing_clone.set(false);
                            status_message_clone.set("Parsing completed successfully!".to_string());
                            log_messages_clone.write().push("Parse process finished. Matrix extracted.".to_string());
                            log_messages_clone.write().push(format!("Extracted matrix dimensions: {:?}", matrix.shape()));
                            log_messages_clone.write().push(format_save_confirmation(output_path_opt.as_deref()));

                            extracted_matrix.set(Some((
                                matrix.nrows(),
                                matrix.ncols(),
                                matrix.iter().copied().collect()
                            )));
                        }
                        Err(e) => {
                            is_parsing_clone.set(false);
                            let err_msg = format!("❌ HiC parsing failed: {}", e);
                            status_message_clone.set(err_msg.clone());
                            log_messages_clone.write().push(err_msg);
                        }
                    }
                }

                //-----------------------------------------------------------------------------
                // COOL parsing branch
                //-----------------------------------------------------------------------------
                ParserFormat::Cool => {
                    // Clone only COOL-specific values here (avoid unused allocs)
                    let chromosome_val = chromosome.read().as_str().to_owned();
                    let resolution_val = resolution.read().as_str().to_owned();
                    let norm_type_val = normalization_type.read().as_str().to_owned();
                    let start_bin_val = start_bin.read().as_str().to_owned();
                    let end_bin_val = end_bin.read().as_str().to_owned();
                    let cool_group_val = cool_group.read().as_str().to_owned();

                    log_messages_clone.write().push("📦 Parsing COOL file...".to_string());
                    status_message_clone.set("Processing .cool format...".to_string());

                    let cool_config = CoolConfig {
                        resolution: resolution_val.parse().unwrap_or(1_000_000),
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
                            String::from("resolutions")
                        } else {
                            cool_group_val
                        },
                    };

                    log_messages_clone.write().push(format!(
                        "Config: res={}, group='{}', chr={:?}",
                        cool_config.resolution, cool_config.group_path, cool_config.chromosome
                    ));

                    let output_path_opt = if !output_path_val.is_empty() {
                        Some(Path::new(&output_path_val))
                    } else {
                        None
                    };

                    log_messages_clone.write().push(format_output_intent(output_path_opt.as_deref()));

                    log_messages_clone.write().push("🚀 Calling extract_cool()...".to_string());

                    match extract_cool(&input_path_val, &cool_config, output_path_opt) {
                        Ok(matrix) => {
                            is_parsing_clone.set(false);
                            status_message_clone.set("✅ COOL parsing completed!".to_string());
                            log_messages_clone.write().push("📊 Parse process finished. Matrix extracted.".to_string());

                            for msg in format_matrix_stats(&matrix) {
                                log_messages_clone.write().push(msg);
                            }
                            log_messages_clone.write().push(format_save_confirmation(output_path_opt.as_deref()));

                            extracted_matrix.set(Some((
                                matrix.nrows(),
                                matrix.ncols(),
                                matrix.iter().copied().collect()
                            )));
                        }
                        Err(e) => {
                            is_parsing_clone.set(false);
                            let err_msg = format!("❌ COOL parsing failed: {}", e);
                            status_message_clone.set(err_msg.clone());
                            log_messages_clone.write().push(err_msg);
                            log_messages_clone.write().push(format!("🔍 Error type: {:?}", e));
                        }
                    }
                }
            }
        });
    };

    // Browse button for INPUT file.
    let on_browse_input = move |_| {
        let mut input_path = input_path;
        spawn(async move {
            if let Some(path) = pick_genomic_input().await {
                input_path.set(path.to_string_lossy().into_owned());
            }
        });
    };

    let on_browse_save = move |_: MouseEvent| {
        let mut output_path = output_path;
        spawn(async move {
            if let Some(path) = pick_serialization_output().await {
                output_path.set(path.to_string_lossy().into_owned());
            }
        });
    };

    // ----------------------------------------------------------------------------------------
    // Render format-specific parameters (dynamic UI)
    // ----------------------------------------------------------------------------------------
    let render_format_params = move || match file_format() {
        ParserFormat::Hic => rsx! {
            div { class: "control-group",
                label { "Chromosome:" }
                input {
                    r#type: "text",
                    value: "{chromosome}",
                    disabled: is_parsing(),
                    oninput: move |evt| chromosome.set(evt.value()),
                }
            }
            div { class: "control-group",
                label { "Resolution (bins):" }
                input {
                    r#type: "text",
                    value: "{resolution}",
                    disabled: is_parsing(),
                    oninput: move |evt| resolution.set(evt.value()),
                }
            }
            div { class: "control-group",
                label { "Start Bin (optional):" }
                input {
                    r#type: "text",
                    value: "{start_bin}",
                    disabled: is_parsing(),
                    oninput: move |evt| start_bin.set(evt.value()),
                }
            }
            div { class: "control-group",
                label { "End Bin (optional):" }
                input {
                    r#type: "text",
                    value: "{end_bin}",
                    disabled: is_parsing(),
                    oninput: move |evt| end_bin.set(evt.value()),
                }
            }
            div { class: "control-group",
                label { "Normalization:" }
                select {
                    value: "{normalization_type}",
                    disabled: is_parsing(),
                    oninput: move |evt| normalization_type.set(evt.value()),
                    option { value: "NONE", "None" }
                    option { value: "VC", "Vanilla Coverage (VC)" }
                    option { value: "VCSQRT", "Vanilla Coverage Sqrt (VCSQRT)" }
                    option { value: "KR", "Knight-Ruiz (KR)" }
                }
            }
        },
        ParserFormat::Cool => rsx! {
            div { class: "control-group",
                label { "Group (e.g., 'resolutions'):" }
                input {
                    r#type: "text",
                    value: "{cool_group}",
                    placeholder: "resolutions",
                    disabled: is_parsing(),
                    oninput: move |evt| cool_group.set(evt.value()),
                }
            }
            div { class: "control-group",
                label { "Resolution:" }
                input {
                    r#type: "text",
                    value: "{resolution}",
                    disabled: is_parsing(),
                    oninput: move |evt| resolution.set(evt.value()),
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
    };

    //-----------------------------------------------------------------------------------------
    // UI Rendering (synchronous, outside spawn)
    //-----------------------------------------------------------------------------------------
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

                    // Format selector (first parameter)
                    div { class: "control-group",
                        label { "File Format:" }
                        select {
                            value: "{file_format().as_str()}",
                            disabled: is_parsing(),
                            oninput: move |evt| file_format.set(ParserFormat::from_str(&evt.value())),
                            option { value: "HIC", "HiC (.hic)" }
                            option { value: "COOL", "Cooler (.cool)" }
                        }
                    }

                    // Common parameters (all formats)
                    div { class: "control-group",
                        label { "Input File:" }
                        div { class: "input-with-button",
                            input {
                                r#type: "text",
                                value: "{input_path()}",
                                placeholder: {
                                    match file_format() {
                                        ParserFormat::Hic => "/home/path/to/yourfile.hic",
                                        ParserFormat::Cool => "/home/path/to/yourfile.cool",
                                    }
                                },
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

                    // Output Directory field (with browse button)
                    div { class: "control-group",
                        label { "Save Output File:" }
                        div { class: "input-with-button",
                            input {
                                r#type: "text",
                                value: "{output_path()}",
                                placeholder: "/home/save/to/path.csv",
                                disabled: is_parsing(),
                                oninput: move |evt| output_path.set(evt.value()),
                            }
                            button {
                                class: "browse-btn",
                                disabled: is_parsing(),
                                onclick: on_browse_save,
                                "📂 Browse..."
                            }
                        }
                    }

                    // Format-specific parameters (dynamic)
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
                    div { class: "status-message", {status_message()} }
                    div { class: "progress-container",
                        div {
                            class: "progress-bar",
                            style: {format!("width: {}%;", progress() * 100.0)},
                        }
                    }
                    p { progress_text }
                    div { class: "parser-logs",
                        h4 { "Log Output" }
                        {
                            for log_line in log_messages.read().iter() {
                                p { log_line }
                            }
                        }
                    }
                }

                // Matrix editor block
                {
                    extracted_matrix
                        // Extract to plain usize to prevent macro/type confusion with references
                        .read()
                        .as_ref()
                        .map(|(rows, cols, mat)| {
                            let limit = *display_limit.read();
                            let r_rows = *rows;
                            let r_cols = *cols;
                            rsx! {
                                div { class: "matrix-editor-panel",
                                    h3 { "📊 Extracted Matrix Preview" }
                                    div { class: "editor-toolbar",
                                        span {
                                            {
                                                format!(
                                                    "Showing first {}x{} of {}x{}",
                                                    limit.min(r_rows),
                                                    limit.min(r_cols),
                                                    r_rows,
                                                    r_cols,
                                                )
                                            }
                                        }
                                        input {
                                            r#type: "number",
                                            class: "limit-input",
                                            value: {limit},
                                            min: "5",
                                            max: "200",
                                            oninput: move |evt| {
                                                if let Ok(v) = evt.value().parse::<usize>() {
                                                    display_limit.set(v.min(200));
                                                }
                                            },
                                        }
                                        button {
                                            class: "save-matrix-btn",
                                            disabled: extracted_matrix.read().is_none(),
                                            onclick: on_save_result,
                                            "💾 Save Full Matrix..."
                                        }
                                    }
                                    div { class: "matrix-grid-container",

                            }
                        })
                }
            }
        }
    }
}

