//! doson - DataValue Parser
//!
//! ```
//! use doson::DataValue;
//! use std::collections::HashMap;
//!
//! DataValue::String("Hello World".to_string());
//! DataValue::Number(1_f64);
//! DataValue::Boolean(true);
//! DataValue::List(vec![
//!     DataValue::Number(3.14_f64),
//! ]);
//! DataValue::Dict(HashMap::new());
//! DataValue::Tuple((
//!     Box::new(DataValue::Number(1_f64)),
//!     Box::new(DataValue::Number(2_f64)),
//! ));
//!
//! ```

mod binary;

use binary::Binary;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use nom::{IResult, branch::alt, bytes::complete::{escaped, tag, tag_no_case, take_till1, take_while_m_n}, character::complete::{alphanumeric0, multispace0}, combinator::{map, peek, value as n_value}, error::context, multi::separated_list0, number::complete::double, sequence::{delimited, preceded, separated_pair}};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataValue {
    /// None Value
    ///
    /// Just use for deserialize.
    None,

    /// String Value
    ///
    /// ```
    /// use doson::DataValue;
    /// DataValue::String("hello world".to_string());
    /// ```
    String(String),

    /// Number Value
    ///
    /// ```
    /// use doson::DataValue;
    /// DataValue::Number(10_f64);
    /// ```
    Number(f64),

    /// Boolean Value
    ///
    /// ```
    /// use doson::DataValue;
    /// DataValue::Boolean(true);
    /// ```
    Boolean(bool),

    /// List Value
    ///
    /// ```
    /// use doson::DataValue;
    ///     DataValue::List(vec![
    ///     DataValue::Number(1.0),
    ///     DataValue::Number(2.0),
    ///     DataValue::Number(3.0)
    /// ]);
    /// ```
    List(Vec<DataValue>),

    /// Dict Value
    ///
    /// ```
    /// use doson::DataValue;
    /// DataValue::Dict(std::collections::HashMap::new());
    /// ```
    Dict(HashMap<String, DataValue>),

    /// Tuple Value
    ///
    /// ```
    /// use doson::DataValue;
    /// DataValue::Tuple(
    ///     (
    ///         Box::new(DataValue::Boolean(true)),
    ///         Box::new(DataValue::Boolean(false))
    ///     )
    /// );
    /// ```
    Tuple((Box<DataValue>, Box<DataValue>)),

    Binary(binary::Binary),
}

impl std::string::ToString for DataValue {
    fn to_string(&self) -> String {
        match self {
            DataValue::None => "none".to_string(),
            DataValue::String(s) => format!("\"{}\"", s),
            DataValue::Number(n) => n.to_string(),
            DataValue::Boolean(bool) => match bool {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            DataValue::List(l) => {
                let mut res = String::from("[");

                for v in l {
                    res += &format!("{},", v.to_string());
                }

                if res.len() > 1 { res = res[..res.len() - 1].to_string(); }

                res += "]";

                res
            }
            DataValue::Dict(d) => {
                let mut res = String::from("{");

                for v in d {
                    res += &format!("\"{}\":{},", v.0, v.1.to_string());
                }

                if res.len() > 1 { res = res[..res.len() - 1].to_string(); }

                res += "}";

                res
            }
            DataValue::Tuple(v) => {
                let first = v.0.to_string();
                let second = v.1.to_string();

                format!("({},{})", first, second)
            },
            DataValue::Binary(v) => {
                v.to_string()
            }
        }
    }
}

impl std::cmp::Ord for DataValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight().partial_cmp(&other.weight()).unwrap_or(Ordering::Equal)
    }
}

impl std::cmp::PartialOrd for DataValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for DataValue {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl std::cmp::Eq for DataValue {}

impl DataValue {

    /// parse `&str` to `DataValue` type:
    /// - String: "xxx"
    /// - Number: 114514
    /// - Boolean: true
    /// - List: \[1,2,3,4,5\]
    /// - dict: {"hello": "world"}
    /// - tuple: (1,2)
    /// ```
    /// use doson::DataValue;
    ///
    /// assert_eq!(
    ///     DataValue::from("[1,2,3]"),
    ///     DataValue::List(vec![
    ///         DataValue::Number(1_f64),
    ///         DataValue::Number(2_f64),
    ///         DataValue::Number(3_f64),
    ///     ])
    /// );
    /// ```
    pub fn from(data: &str) -> Self {

        let mut data = data.to_string();
        if data.len() >= 3 {
            if &data[0..2] == "b:" && &data[data.len() - 1..] == ":" {
                let temp = &data[2 .. data.len() - 1];
                let temp = base64::decode(temp).unwrap_or(vec![]);
                data = String::from_utf8(temp).unwrap_or(String::new());
            }
        }

        match ValueParser::parse(&data) {
            Ok((_, v)) => v,
            Err(_) => Self::None,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or(String::from("None"))
    }

    // 数据权值计算
    // Number(f64) 的权值等于它本身
    // 其他基本类型的权值为 f64::MAX
    // 复合类型则会进行递归计算
    // 权值主要用于排序等操作
    pub fn weight(&self) -> f64 {

        if let DataValue::Number(n) = self {
            return *n;
        }

        // 计算数组的权重值
        if let DataValue::List(l) = self {
            let mut total = 0_f64;
            for item in l {
                let mut temp = item.weight();
                if temp == f64::MAX { temp = 0_f64; }
                total += temp;
            }
            return total;
        }

        if let DataValue::Dict(d) = self {
            let mut total = 0_f64;
            for (_, item) in d {
                let mut temp = item.weight();
                if temp == f64::MAX { temp = 0_f64; }
                total += temp;
            }
            return total;
        }

        if let DataValue::Tuple(v) = self {
            let mut total = 0_f64;

            // 元组值0
            let mut temp = v.0.weight();
            if temp == f64::MAX { temp = 0_f64; }
            total += temp;

            // 元组值1
            let mut temp = v.1.weight();
            if temp == f64::MAX { temp = 0_f64; }
            total += temp;

            return total;
        }

        f64::MAX
    }

    pub fn size(&self) -> usize {
        match self {
            DataValue::None => 0,
            DataValue::String(str) => str.len(),
            DataValue::Number(_) => 8,
            DataValue::Boolean(_) => 1,

            DataValue::List(list) => {
                let mut result = 0;

                for item in list {
                    result += item.size();
                }

                result
            }
            DataValue::Dict(dict) => {
                let mut result = 0;

                for item in dict {
                    result += item.1.size();
                }

                result
            }

            DataValue::Tuple(tuple) => tuple.0.size() + tuple.1.size(),
            DataValue::Binary(bin) => bin.size(),
        }
    }

    pub fn datatype(&self) -> String {
        return match self {
            DataValue::None => "None",
            DataValue::String(_) => "String",
            DataValue::Number(_) => "Number",
            DataValue::Boolean(_) => "Boolean",
            DataValue::List(_) => "List",
            DataValue::Dict(_) => "Dict",
            DataValue::Tuple(_) => "Tuple",
            DataValue::Binary(_) => "Binary",
        }.to_string();
    }

    pub fn as_string(&self) -> Option<String> {
        return match self {
            DataValue::String(val) => Some(val.to_string()),
            _ => None
        }
    }


    pub fn as_number(&self) -> Option<f64> {
        return match self {
            DataValue::Number(val) => Some(*val),
            _ => None
        }
    }


    pub fn as_bool(&self) -> Option<bool> {
        return match self {
            DataValue::Boolean(val) => Some(*val),
            _ => None
        }
    }

    pub fn as_tuple(&self) -> Option<(Box<DataValue>, Box<DataValue>)> {
        return match self {
            DataValue::Tuple(val) => Some(val.clone()),
            _ => None
        }
    }

    pub fn as_list(&self) -> Option<Vec<DataValue>> {
        return match self {
            DataValue::List(val) => Some(val.clone()),
            _ => None
        }
    }

    pub fn as_dict(&self) -> Option<HashMap<String, DataValue>> {
        return match self {
            DataValue::Dict(val) => Some(val.clone()),
            _ => None
        }
    }

}

struct ValueParser {}
impl ValueParser {

    fn normal(message: &str) -> IResult<&str, &str> {
        take_till1(|c: char| c == '\\' || c == '"' || c.is_ascii_control())(message)
    }

    fn escapable(i: &str) -> IResult<&str, &str> {
        context(
            "escaped",
            alt((
                tag("\""),
                tag("\\"),
                tag("/"),
                tag("b"),
                tag("f"),
                tag("n"),
                tag("r"),
                tag("t"),
                ValueParser::parse_hex,
            )),
        )(i)
    }

    fn string_format(message: &str) -> IResult<&str, &str> {
        escaped(ValueParser::normal, '\\', ValueParser::escapable)(message)
    }

    fn parse_hex(message: &str) -> IResult<&str, &str> {
        context(
            "hex string",
            preceded(
                peek(tag("u")),
                take_while_m_n(5, 5, |c: char| c.is_ascii_hexdigit() || c == 'u'),
            ),
        )(message)
    }

    fn parse_string(message: &str) -> IResult<&str, &str> {
        context(
            "string",
            alt((
                tag("\"\""),
                delimited(tag("\""), ValueParser::string_format, tag("\"")),
            )),
        )(message)
    }

    fn parse_binary(message: &str) -> IResult<&str, Binary> {
        
        let result: (&str, &str) = context(
            "binary", 
            alt((
                tag("binary!()"),
                delimited(tag("binary!("), alphanumeric0, tag(")"))
            ))
        )(message)?;

        Ok((
            result.0, 
            Binary::from_b64(result.1.to_string()).unwrap_or(Binary::build(vec![]))
        ))
    }

    fn parse_number(message: &str) -> IResult<&str, f64> {
        double(message)
    }

    fn parse_boolean(message: &str) -> IResult<&str, bool> {
        let parse_true = n_value(true, tag_no_case("true"));
        let parse_false = n_value(false, tag_no_case("false"));
        alt((parse_true, parse_false))(message)
    }

    fn parse_list(message: &str) -> IResult<&str, Vec<DataValue>> {
        context(
            "list",
            delimited(
                tag("["),
                separated_list0(
                    tag(","),
                    delimited(multispace0, ValueParser::parse, multispace0),
                ),
                tag("]"),
            ),
        )(message)
    }

    fn parse_dict(message: &str) -> IResult<&str, HashMap<String, DataValue>> {
        context(
            "object",
            delimited(
                tag("{"),
                map(
                    separated_list0(
                        tag(","),
                        separated_pair(
                            delimited(multispace0, ValueParser::parse_string, multispace0),
                            tag(":"),
                            delimited(multispace0, ValueParser::parse, multispace0),
                        ),
                    ),
                    |tuple_vec: Vec<(&str, DataValue)>| {
                        tuple_vec
                            .into_iter()
                            .map(|(k, v)| (String::from(k), v))
                            .collect()
                    },
                ),
                tag("}"),
            ),
        )(message)
    }

    fn parse_tuple(message: &str) -> IResult<&str, (Box<DataValue>, Box<DataValue>)> {
        context(
            "tuple",
            delimited(
                tag("("),
                map(
                    separated_pair(
                        delimited(multispace0, ValueParser::parse, multispace0),
                        tag(","),
                        delimited(multispace0, ValueParser::parse, multispace0),
                    ),
                    |pair: (DataValue, DataValue)| (Box::new(pair.0), Box::new(pair.1)),
                ),
                tag(")"),
            ),
        )(message)
    }

    fn parse(message: &str) -> IResult<&str, DataValue> {

        context(
            "value",
            delimited(
                multispace0,
                alt((
                    map(ValueParser::parse_number, DataValue::Number),
                    map(ValueParser::parse_boolean, DataValue::Boolean),
                    map(ValueParser::parse_string, |s| {
                        DataValue::String(String::from(s))
                    }),
                    map(ValueParser::parse_list, DataValue::List),
                    map(ValueParser::parse_dict, DataValue::Dict),
                    map(ValueParser::parse_tuple, DataValue::Tuple),
                    map(ValueParser::parse_binary, DataValue::Binary)
                )),
                multispace0,
            ),
        )(&message)
    }
}

#[cfg(test)]
mod test {

    use crate::{DataValue, ValueParser};

    #[test]
    fn list() {
        let value = "[1, 2, 3, 4, 5, 6]";
        assert_eq!(
            ValueParser::parse(value),
            Ok((
                "",
                DataValue::List(vec![
                    DataValue::Number(1_f64),
                    DataValue::Number(2_f64),
                    DataValue::Number(3_f64),
                    DataValue::Number(4_f64),
                    DataValue::Number(5_f64),
                    DataValue::Number(6_f64),
                ])
            ))
        );
    }

    #[test]
    fn tuple() {
        let value = "(true,1)";
        assert_eq!(
            ValueParser::parse(value),
            Ok((
                "",
                DataValue::Tuple((
                    Box::new(DataValue::Boolean(true)),
                    Box::new(DataValue::Number(1_f64))
                ))
            ))
        );
    }

    #[test]
    fn binary() {
        let message = "binary!(DOREASERVERTEST)";
        assert_eq!(
            message,
            "DOREASERVERTEST",
        )
    }

    #[test]
    fn to_json() {
        let value = DataValue::List(vec![
            DataValue::Number(1.0),
            DataValue::Number(2.0),
            DataValue::Number(3.0)
        ]);
        
        assert_eq!(
            value.to_json(),
            String::from("{\"List\":[{\"Number\":1.0},{\"Number\":2.0},{\"Number\":3.0}]}")
        )
    }
}
