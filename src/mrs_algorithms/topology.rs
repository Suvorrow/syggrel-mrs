use pyo3::prelude::*;
use pyo3::types::PyModule;
use numpy::PyArray2;
use std::ffi::CString;
use crate::types::GenomicTarget;
use crate::mrs_algorithms::data_integration::load_contact_matrix;

pub fn compute_topological_risk(target: &GenomicTarget) -> f64 {
    Python::with_gil(|py| {
        let python_code = CString::new(
            r#"
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
        "#).unwrap();

        let gudhi_code = PyModule::from_code(
            py,
            &python_code,
            &CString::new("gudhi_module").unwrap(),
            &CString::new("gudhi_module").unwrap()
        ).unwrap();

        // Get the contact matrix for the target
        let contact_matrix = load_contact_matrix(target);

        // Convert ndarray to Pyhton object
        let py_matrix = PyArray2::from_owned_array(py, contact_matrix);

        let result = gudhi_code.call_method1("compute_chromatin_topology", (py_matrix,)).unwrap();
        result.extract::<f64>().unwrap()
    })
}