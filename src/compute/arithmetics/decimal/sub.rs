// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Defines the subtract arithmetic kernels for Decimal `PrimitiveArrays`.

use crate::compute::arithmetics::basic::check_same_len;
use crate::{
    array::{Array, PrimitiveArray},
    buffer::Buffer,
    compute::{
        arithmetics::{ArrayCheckedSub, ArraySaturatingSub, ArraySub},
        arity::{binary, binary_checked},
        utils::combine_validities,
    },
    datatypes::DataType,
    error::{ArrowError, Result},
};

use super::{adjusted_precision_scale, max_value, number_digits};

/// Subtract two decimal primitive arrays with the same precision and scale. If
/// the precision and scale is different, then an InvalidArgumentError is
/// returned. This function panics if the subtracted numbers result in a number
/// smaller than the possible number for the selected precision.
///
/// # Examples
/// ```
/// use arrow2::compute::arithmetics::decimal::sub::sub;
/// use arrow2::array::PrimitiveArray;
/// use arrow2::datatypes::DataType;
///
/// let a = PrimitiveArray::from([Some(1i128), Some(1i128), None, Some(2i128)]).to(DataType::Decimal(5, 2));
/// let b = PrimitiveArray::from([Some(1i128), Some(2i128), None, Some(2i128)]).to(DataType::Decimal(5, 2));
///
/// let result = sub(&a, &b).unwrap();
/// let expected = PrimitiveArray::from([Some(0i128), Some(-1i128), None, Some(0i128)]).to(DataType::Decimal(5, 2));
///
/// assert_eq!(result, expected);
/// ```
pub fn sub(lhs: &PrimitiveArray<i128>, rhs: &PrimitiveArray<i128>) -> Result<PrimitiveArray<i128>> {
    // Matching on both data types from both arrays This match will be true
    // only when precision and scale from both arrays are the same, otherwise
    // it will return and ArrowError
    match (lhs.data_type(), rhs.data_type()) {
        (DataType::Decimal(lhs_p, lhs_s), DataType::Decimal(rhs_p, rhs_s)) => {
            if lhs_p == rhs_p && lhs_s == rhs_s {
                // Closure for the binary operation. This closure will panic if
                // the sum of the values is larger than the max value possible
                // for the decimal precision
                let op = move |a, b| {
                    let res: i128 = a - b;

                    if res.abs() > max_value(*lhs_p) {
                        panic!("Overflow in subtract presented for precision {}", lhs_p);
                    }

                    res
                };

                binary(lhs, rhs, lhs.data_type().clone(), op)
            } else {
                Err(ArrowError::InvalidArgumentError(
                    "Arrays must have the same precision and scale".to_string(),
                ))
            }
        }
        _ => Err(ArrowError::InvalidArgumentError(
            "Incorrect data type for the array".to_string(),
        )),
    }
}

/// Saturated subtraction of two decimal primitive arrays with the same
/// precision and scale. If the precision and scale is different, then an
/// InvalidArgumentError is returned. If the result from the sum is smaller
/// than the possible number with the selected precision then the resulted
/// number in the arrow array is the minimum number for the selected precision.
///
/// # Examples
/// ```
/// use arrow2::compute::arithmetics::decimal::sub::saturating_sub;
/// use arrow2::array::PrimitiveArray;
/// use arrow2::datatypes::DataType;
///
/// let a = PrimitiveArray::from([Some(-99000i128), Some(11100i128), None, Some(22200i128)]).to(DataType::Decimal(5, 2));
/// let b = PrimitiveArray::from([Some(01000i128), Some(22200i128), None, Some(11100i128)]).to(DataType::Decimal(5, 2));
///
/// let result = saturating_sub(&a, &b).unwrap();
/// let expected = PrimitiveArray::from([Some(-99999i128), Some(-11100i128), None, Some(11100i128)]).to(DataType::Decimal(5, 2));
///
/// assert_eq!(result, expected);
/// ```
pub fn saturating_sub(
    lhs: &PrimitiveArray<i128>,
    rhs: &PrimitiveArray<i128>,
) -> Result<PrimitiveArray<i128>> {
    // Matching on both data types from both arrays. This match will be true
    // only when precision and scale from both arrays are the same, otherwise
    // it will return and ArrowError
    match (lhs.data_type(), rhs.data_type()) {
        (DataType::Decimal(lhs_p, lhs_s), DataType::Decimal(rhs_p, rhs_s)) => {
            if lhs_p == rhs_p && lhs_s == rhs_s {
                // Closure for the binary operation.
                let op = move |a, b| {
                    let res: i128 = a - b;
                    let max: i128 = max_value(*lhs_p);

                    match res {
                        res if res.abs() > max => {
                            if res > 0 {
                                max
                            } else {
                                -max
                            }
                        }
                        _ => res,
                    }
                };

                binary(lhs, rhs, lhs.data_type().clone(), op)
            } else {
                Err(ArrowError::InvalidArgumentError(
                    "Arrays must have the same precision and scale".to_string(),
                ))
            }
        }
        _ => Err(ArrowError::InvalidArgumentError(
            "Incorrect data type for the array".to_string(),
        )),
    }
}

// Implementation of ArraySub trait for PrimitiveArrays
impl ArraySub<PrimitiveArray<i128>> for PrimitiveArray<i128> {
    type Output = Self;

    fn sub(&self, rhs: &PrimitiveArray<i128>) -> Result<Self::Output> {
        sub(self, rhs)
    }
}

// Implementation of ArrayCheckedSub trait for PrimitiveArrays
impl ArrayCheckedSub<PrimitiveArray<i128>> for PrimitiveArray<i128> {
    type Output = Self;

    fn checked_sub(&self, rhs: &PrimitiveArray<i128>) -> Result<Self::Output> {
        checked_sub(self, rhs)
    }
}

// Implementation of ArraySaturatingSub trait for PrimitiveArrays
impl ArraySaturatingSub<PrimitiveArray<i128>> for PrimitiveArray<i128> {
    type Output = Self;

    fn saturating_sub(&self, rhs: &PrimitiveArray<i128>) -> Result<Self::Output> {
        saturating_sub(self, rhs)
    }
}
/// Checked subtract of two decimal primitive arrays with the same precision
/// and scale. If the precision and scale is different, then an
/// InvalidArgumentError is returned. If the result from the sub is larger than
/// the possible number with the selected precision (overflowing), then the
/// validity for that index is changed to None
///
/// # Examples
/// ```
/// use arrow2::compute::arithmetics::decimal::sub::checked_sub;
/// use arrow2::array::PrimitiveArray;
/// use arrow2::datatypes::DataType;
///
/// let a = PrimitiveArray::from([Some(-99000i128), Some(11100i128), None, Some(22200i128)]).to(DataType::Decimal(5, 2));
/// let b = PrimitiveArray::from([Some(01000i128), Some(22200i128), None, Some(11100i128)]).to(DataType::Decimal(5, 2));
///
/// let result = checked_sub(&a, &b).unwrap();
/// let expected = PrimitiveArray::from([None, Some(-11100i128), None, Some(11100i128)]).to(DataType::Decimal(5, 2));
///
/// assert_eq!(result, expected);
/// ```
pub fn checked_sub(
    lhs: &PrimitiveArray<i128>,
    rhs: &PrimitiveArray<i128>,
) -> Result<PrimitiveArray<i128>> {
    // Matching on both data types from both arrays. This match will be true
    // only when precision and scale from both arrays are the same, otherwise
    // it will return and ArrowError
    match (lhs.data_type(), rhs.data_type()) {
        (DataType::Decimal(lhs_p, lhs_s), DataType::Decimal(rhs_p, rhs_s)) => {
            if lhs_p == rhs_p && lhs_s == rhs_s {
                // Closure for the binary operation.
                let op = move |a, b| {
                    let res: i128 = a - b;

                    match res {
                        res if res.abs() > max_value(*lhs_p) => None,
                        _ => Some(res),
                    }
                };

                binary_checked(lhs, rhs, lhs.data_type().clone(), op)
            } else {
                Err(ArrowError::InvalidArgumentError(
                    "Arrays must have the same precision and scale".to_string(),
                ))
            }
        }
        _ => Err(ArrowError::InvalidArgumentError(
            "Incorrect data type for the array".to_string(),
        )),
    }
}

/// Adaptive subtract of two decimal primitive arrays with different precision
/// and scale. If the precision and scale is different, then the smallest scale
/// and precision is adjusted to the largest precision and scale. If during the
/// addition one of the results is smaller than the min possible value, the
/// result precision is changed to the precision of the min value
///
/// ```nocode
///  99.9999 -> 6, 4
/// -00.0001 -> 6, 4
/// -----------------
/// 100.0000 -> 7, 4
/// ```
/// # Examples
/// ```
/// use arrow2::compute::arithmetics::decimal::sub::adaptive_sub;
/// use arrow2::array::PrimitiveArray;
/// use arrow2::datatypes::DataType;
///
/// let a = PrimitiveArray::from([Some(99_9999i128)]).to(DataType::Decimal(6, 4));
/// let b = PrimitiveArray::from([Some(-00_0001i128)]).to(DataType::Decimal(6, 4));
/// let result = adaptive_sub(&a, &b).unwrap();
/// let expected = PrimitiveArray::from([Some(100_0000i128)]).to(DataType::Decimal(7, 4));
///
/// assert_eq!(result, expected);
/// ```
pub fn adaptive_sub(
    lhs: &PrimitiveArray<i128>,
    rhs: &PrimitiveArray<i128>,
) -> Result<PrimitiveArray<i128>> {
    check_same_len(lhs, rhs)?;

    if let (DataType::Decimal(lhs_p, lhs_s), DataType::Decimal(rhs_p, rhs_s)) =
        (lhs.data_type(), rhs.data_type())
    {
        // The resulting precision is mutable because it could change while
        // looping through the iterator
        let (mut res_p, res_s, diff) = adjusted_precision_scale(*lhs_p, *lhs_s, *rhs_p, *rhs_s);

        let mut result = Vec::new();
        for (l, r) in lhs.values().iter().zip(rhs.values().iter()) {
            // Based on the array's scales one of the arguments in the sum has to be shifted
            // to the left to match the final scale
            let res: i128 = if lhs_s > rhs_s {
                l - r * 10i128.pow(diff as u32)
            } else {
                l * 10i128.pow(diff as u32) - r
            };

            // The precision of the resulting array will change if one of the
            // subtraction during the iteration produces a value bigger than the
            // possible value for the initial precision

            //  -99.9999 -> 6, 4
            //   00.0001 -> 6, 4
            // -----------------
            // -100.0000 -> 7, 4
            if res.abs() > max_value(res_p) {
                res_p = number_digits(res);
            }

            result.push(res);
        }

        let validity = combine_validities(lhs.validity(), rhs.validity());
        let values = Buffer::from(result);

        Ok(PrimitiveArray::<i128>::from_data(
            DataType::Decimal(res_p, res_s),
            values,
            validity,
        ))
    } else {
        Err(ArrowError::InvalidArgumentError(
            "Incorrect data type for the array".to_string(),
        ))
    }
}
