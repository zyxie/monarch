/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use cxx::ExternType;
use cxx::type_id;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::ScalarType;
use crate::bridge::ffi;

// Bind generated types to CXX
// SAFETY: This type is trival, just an i8.
unsafe impl ExternType for ScalarType {
    type Id = type_id!("c10::ScalarType");
    type Kind = cxx::kind::Trivial;
}

impl ScalarType {
    pub(crate) fn from_py_object_or_none(obj: &Bound<'_, PyAny>) -> Option<Self> {
        ffi::py_object_is_scalar_type(obj.clone().into())
            .then(|| ffi::scalar_type_from_py_object(obj.into()).unwrap())
    }
}

impl FromPyObject<'_> for ScalarType {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        ffi::scalar_type_from_py_object(obj.into()).map_err(|e| {
            PyValueError::new_err(format!(
                "Failed extracting {} from py as ScalarType: {}",
                obj, e
            ))
        })
    }
}

impl<'py> IntoPyObject<'py> for ScalarType {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        ffi::scalar_type_to_py_object(self).into_pyobject(py)
    }
}

// Remotely implement Serialize/Deserialize for generated types
// TODO: we should be able to use parse_callbacks + add_derives, (see
// https://github.com/rust-lang/rust-bindgen/pull/2059) and avoid a remote
#[derive(Serialize, Deserialize)]
#[serde(remote = "ScalarType")]
pub struct ScalarTypeDef(i8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_py_and_back() {
        pyo3::prepare_freethreaded_python();
        let scalar_type = ScalarType::Float;
        let converted_type = Python::with_gil(|py| {
            // import torch to ensure torch.dtype types are registered
            py.import("torch").unwrap();
            let obj = scalar_type.into_pyobject(py).unwrap();
            obj.extract::<ScalarType>().unwrap()
        });
        assert_eq!(converted_type, ScalarType::Float);
    }

    #[test]
    fn from_py() {
        pyo3::prepare_freethreaded_python();
        let scalar_type = Python::with_gil(|py| {
            let obj = py.import("torch").unwrap().getattr("float32").unwrap();
            obj.extract::<ScalarType>().unwrap()
        });
        assert_eq!(scalar_type, ScalarType::Float);
    }
}
