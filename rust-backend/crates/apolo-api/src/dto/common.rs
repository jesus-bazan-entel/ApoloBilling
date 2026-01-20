//! Common DTOs used across the API

use apolo_core::traits::{PaginatedResponse, PaginationMeta};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,
    /// Response message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a success response with data
    pub fn success(data: T) -> Self {
        Self {
            data,
            message: None,
        }
    }

    /// Create a success response with data and message
    pub fn with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            data,
            message: Some(message.into()),
        }
    }
}

/// Pagination query parameters
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    #[validate(range(min = 1))]
    pub page: i64,

    /// Items per page
    #[serde(default = "default_per_page")]
    #[validate(range(min = 1, max = 1000))]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    50
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

impl PaginationParams {
    /// Calculate offset for database query
    #[inline]
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }

    /// Get limit for database query
    #[inline]
    pub fn limit(&self) -> i64 {
        self.per_page
    }

    /// Create pagination metadata
    pub fn metadata(&self, total: i64) -> PaginationMeta {
        PaginationMeta::new(total, self.page, self.per_page)
    }

    /// Create paginated response
    pub fn paginate<T>(&self, data: Vec<T>, total: i64) -> PaginatedResponse<T> {
        PaginatedResponse {
            data,
            pagination: self.metadata(total),
        }
    }
}

/// Export format
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// CSV format
    Csv,
    /// JSON format (array of objects)
    Json,
    /// JSON Lines format (one object per line)
    Jsonl,
}

impl ExportFormat {
    /// Get content type header value
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Csv => "text/csv; charset=utf-8",
            Self::Json => "application/json; charset=utf-8",
            Self::Jsonl => "application/x-ndjson; charset=utf-8",
        }
    }

    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Jsonl => "jsonl",
        }
    }
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_offset() {
        let params = PaginationParams {
            page: 1,
            per_page: 10,
        };
        assert_eq!(params.offset(), 0);
        assert_eq!(params.limit(), 10);

        let params = PaginationParams {
            page: 3,
            per_page: 20,
        };
        assert_eq!(params.offset(), 40);
        assert_eq!(params.limit(), 20);
    }

    #[test]
    fn test_export_format() {
        assert_eq!(ExportFormat::Csv.content_type(), "text/csv; charset=utf-8");
        assert_eq!(ExportFormat::Csv.extension(), "csv");
        assert_eq!(
            ExportFormat::Json.content_type(),
            "application/json; charset=utf-8"
        );
    }

    #[test]
    fn test_api_response() {
        let resp = ApiResponse::success("test");
        assert_eq!(resp.data, "test");
        assert!(resp.message.is_none());

        let resp = ApiResponse::with_message("data", "success");
        assert_eq!(resp.message, Some("success".to_string()));
    }
}
