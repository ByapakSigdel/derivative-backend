//! Pagination utilities for list endpoints.

use serde::{Deserialize, Serialize};

/// Default page size
pub const DEFAULT_PAGE_SIZE: u32 = 20;

/// Maximum page size
pub const MAX_PAGE_SIZE: u32 = 100;

/// Pagination parameters
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    DEFAULT_PAGE_SIZE
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: DEFAULT_PAGE_SIZE,
        }
    }
}

impl PaginationParams {
    /// Create new pagination params from optional values
    pub fn new(page: Option<u32>, per_page: Option<u32>) -> Self {
        Self {
            page: page.unwrap_or(1).max(1),
            per_page: per_page.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE),
        }
    }
    
    /// Get the offset for SQL queries
    pub fn offset(&self) -> i64 {
        ((self.page.saturating_sub(1)) * self.per_page) as i64
    }
    
    /// Get the limit for SQL queries
    pub fn limit(&self) -> i64 {
        self.per_page as i64
    }
    
    /// Normalize the pagination params (ensure valid values)
    pub fn normalize(&self) -> Self {
        Self {
            page: self.page.max(1),
            per_page: self.per_page.clamp(1, MAX_PAGE_SIZE),
        }
    }
}

/// Pagination metadata for responses
#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationMeta {
    /// Create pagination metadata from params and total count
    pub fn new(params: &PaginationParams, total: i64) -> Self {
        let total_pages = ((total as f64) / (params.per_page as f64)).ceil() as u32;
        let total_pages = total_pages.max(1);
        
        Self {
            page: params.page,
            per_page: params.per_page,
            total,
            total_pages,
            has_next: params.page < total_pages,
            has_prev: params.page > 1,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, params: &PaginationParams, total: i64) -> Self {
        Self {
            data,
            pagination: PaginationMeta::new(params, total),
        }
    }
}

/// Helper trait for converting query results to paginated responses
pub trait Paginate<T> {
    fn paginate(self, params: &PaginationParams, total: i64) -> PaginatedResponse<T>;
}

impl<T> Paginate<T> for Vec<T> {
    fn paginate(self, params: &PaginationParams, total: i64) -> PaginatedResponse<T> {
        PaginatedResponse::new(self, params, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pagination_offset() {
        let params = PaginationParams::new(Some(1), Some(10));
        assert_eq!(params.offset(), 0);
        
        let params = PaginationParams::new(Some(2), Some(10));
        assert_eq!(params.offset(), 10);
        
        let params = PaginationParams::new(Some(3), Some(20));
        assert_eq!(params.offset(), 40);
    }
    
    #[test]
    fn test_pagination_limit() {
        let params = PaginationParams::new(Some(1), Some(10));
        assert_eq!(params.limit(), 10);
        
        // Test clamping to max
        let params = PaginationParams::new(Some(1), Some(200));
        assert_eq!(params.limit(), MAX_PAGE_SIZE as i64);
    }
    
    #[test]
    fn test_pagination_meta() {
        let params = PaginationParams::new(Some(2), Some(10));
        let meta = PaginationMeta::new(&params, 45);
        
        assert_eq!(meta.page, 2);
        assert_eq!(meta.per_page, 10);
        assert_eq!(meta.total, 45);
        assert_eq!(meta.total_pages, 5);
        assert!(meta.has_next);
        assert!(meta.has_prev);
    }
    
    #[test]
    fn test_pagination_first_page() {
        let params = PaginationParams::new(Some(1), Some(10));
        let meta = PaginationMeta::new(&params, 45);
        
        assert!(!meta.has_prev);
        assert!(meta.has_next);
    }
    
    #[test]
    fn test_pagination_last_page() {
        let params = PaginationParams::new(Some(5), Some(10));
        let meta = PaginationMeta::new(&params, 45);
        
        assert!(meta.has_prev);
        assert!(!meta.has_next);
    }
}
