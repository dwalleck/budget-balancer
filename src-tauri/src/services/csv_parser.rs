use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub date: String,
    pub amount: String,
    pub description: String,
    pub merchant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    pub date: String,
    pub amount: f64,
    pub description: String,
    pub merchant: Option<String>,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String),
    MissingColumn(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO Error: {}", e),
            CsvError::ParseError(e) => write!(f, "Parse Error: {}", e),
            CsvError::MissingColumn(col) => write!(f, "Missing column: {}", col),
        }
    }
}

impl std::error::Error for CsvError {}

pub struct CsvParser;

impl CsvParser {
    /// Normalize date to YYYY-MM-DD format
    fn normalize_date(date_str: &str) -> Result<String, CsvError> {
        use chrono::NaiveDate;

        // Try common date formats
        let formats = [
            "%Y-%m-%d",    // 2025-06-15
            "%m/%d/%Y",    // 06/15/2025
            "%m/%d/%y",    // 06/15/25
            "%Y/%m/%d",    // 2025/06/15
            "%d/%m/%Y",    // 15/06/2025
            "%d-%m-%Y",    // 15-06-2025
            "%b %d, %Y",   // Jun 15, 2025
            "%B %d, %Y",   // June 15, 2025
        ];

        for format in &formats {
            if let Ok(date) = NaiveDate::parse_from_str(date_str.trim(), format) {
                return Ok(date.format("%Y-%m-%d").to_string());
            }
        }

        Err(CsvError::ParseError(format!(
            "Unable to parse date: {}. Supported formats: YYYY-MM-DD, MM/DD/YYYY, etc.",
            date_str
        )))
    }

    pub fn get_headers(csv_content: &str) -> Result<Vec<String>, CsvError> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_content.as_bytes());

        match reader.headers() {
            Ok(headers) => Ok(headers.iter().map(|h| h.to_string()).collect()),
            Err(e) => Err(CsvError::ParseError(e.to_string())),
        }
    }

    pub fn parse(
        csv_content: &str,
        mapping: &ColumnMapping,
    ) -> Result<Vec<ParsedTransaction>, CsvError> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_content.as_bytes());

        let headers = match reader.headers() {
            Ok(h) => h.clone(),
            Err(e) => return Err(CsvError::ParseError(e.to_string())),
        };

        // Verify all required columns exist
        let header_map: HashMap<String, usize> = headers
            .iter()
            .enumerate()
            .map(|(i, h)| (h.to_string(), i))
            .collect();

        if !header_map.contains_key(&mapping.date) {
            return Err(CsvError::MissingColumn(mapping.date.clone()));
        }
        if !header_map.contains_key(&mapping.amount) {
            return Err(CsvError::MissingColumn(mapping.amount.clone()));
        }
        if !header_map.contains_key(&mapping.description) {
            return Err(CsvError::MissingColumn(mapping.description.clone()));
        }

        let mut transactions = Vec::new();

        for result in reader.records() {
            let record = match result {
                Ok(r) => r,
                Err(e) => return Err(CsvError::ParseError(e.to_string())),
            };

            let date_raw = record
                .get(header_map[&mapping.date])
                .ok_or_else(|| CsvError::ParseError("Missing date value".to_string()))?;

            let date = Self::normalize_date(date_raw)?;

            let amount_str = record
                .get(header_map[&mapping.amount])
                .ok_or_else(|| CsvError::ParseError("Missing amount value".to_string()))?;

            // Clean amount string (remove $ and commas)
            let cleaned_amount = amount_str.replace("$", "").replace(",", "");
            let amount: f64 = cleaned_amount
                .parse()
                .map_err(|_| CsvError::ParseError(format!("Invalid amount: {}", amount_str)))?;

            let description = record
                .get(header_map[&mapping.description])
                .ok_or_else(|| CsvError::ParseError("Missing description value".to_string()))?
                .to_string();

            let merchant = mapping.merchant.as_ref().and_then(|m| {
                header_map.get(m).and_then(|&i| record.get(i).map(|s| s.to_string()))
            });

            transactions.push(ParsedTransaction {
                date,
                amount,
                description,
                merchant,
            });
        }

        Ok(transactions)
    }
}
