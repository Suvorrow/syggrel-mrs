use pyo3::prelude::*;
use pyo3::types::PyModule;
use numpy::PyArray2;
use crate::types::GenomicTarget;
use crate::mrs_algorithms::data_integration::load_contact_matrix;

pub fn compute_topological_risk(target: &GenomicTarget) -> f64 {
    // Acquire the GIL and execute the Python code
    Python::attach(|py| -> PyResult<f64> {  // Wrap in PyResult for better error handling
        // Define the Python code as a static string
        const PYTHON_CODE_STR: &str = r#"
import numpy as np
import gudhi as gd

def compute_chromatin_topology(contact_matrix):
    # Convert to numpy array if needed
    contact_matrix = np.array(contact_matrix)

    # Convert contacts to distances
    distance_matrix = 1.0 / (contact_matrix + 1e-8)

    # Create Rips complex
    rips_complex = gd.RipsComplex(distance_matrix=distance_matrix, max_edge_length=2.0)
    simplex_tree = rips_complex.create_simplex_tree(max_dimension=2)

    # Compute persistence
    diag = simplex_tree.persistence()

    # Extract H1 (loops) and H2 (voids) features
    h1_features = [p[1] - p[0] for dim, p in diag if dim == 1 and p[1] != float('inf')]
    h2_features = [p[1] - p[0] for dim, p in diag if dim == 2 and p[1] != float('inf')]

    # Return topological stability metric (more sophisticated than just counting)
    total_persistence = sum(h1_features) + sum(h2_features)
    return float(total_persistence)
"#;

        use std::ffi::CStr;
        let python_code_cstr = CStr::from_bytes_until_nul(PYTHON_CODE_STR.as_bytes())
            .expect("Python code string should not contain interior nulls and should end with \\0");
        
        let module_name_cstr = c"gudhi_module.py";
        let module_name_short_cstr = c"gudhi_module";

        // Create the Python module from the static code string
        let gudhi_module =  PyModule::from_code(
            py,
            python_code_cstr,
            module_name_cstr,
            module_name_short_cstr
        )?; 

        // Get the contact matrix for the target
        let contact_matrix = load_contact_matrix(target);

        // Convert ndarray to Pyhton object
        let py_matrix = PyArray2::from_owned_array(py, contact_matrix);
        
        // Call the Python function with proper error propagation
        let result_py_object = gudhi_module.call_method1("compute_chromatin_topology", (py_matrix,))?;
        
        // Extract the result as f64, propagating any conversion errors
        let result_f64: f64 = result_py_object.extract()?;

        Ok(result_f64)
    }).expect("Python execution failed in compute_topological_risk")
}