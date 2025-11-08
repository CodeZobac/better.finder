/// Calculator provider for mathematical expression evaluation
///
/// This provider detects and evaluates mathematical expressions, supporting:
/// - Basic arithmetic operations (+, -, *, /)
/// - Parentheses and order of operations
/// - Decimal numbers
/// - Common mathematical functions

use crate::error::{LauncherError, Result};
use crate::search::SearchProvider;
use crate::types::{ResultAction, ResultType, SearchResult};
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info};

/// Expression evaluator wrapper around meval
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// Creates a new ExpressionEvaluator
    pub fn new() -> Self {
        Self
    }

    /// Validates if a string is a valid mathematical expression
    pub fn is_valid_expression(expr: &str) -> bool {
        // Check if expression contains only valid characters
        let valid_chars = Regex::new(r"^[\d\s\+\-\*/\(\)\.\^%]+$").unwrap();
        
        if !valid_chars.is_match(expr) {
            return false;
        }

        // Must contain at least one operator or be a number
        let has_operator = expr.contains('+') 
            || expr.contains('-') 
            || expr.contains('*') 
            || expr.contains('/') 
            || expr.contains('^')
            || expr.contains('%');
        
        let is_number = expr.trim().parse::<f64>().is_ok();

        has_operator || is_number
    }

    /// Evaluates a mathematical expression
    pub fn evaluate(expr: &str) -> Result<f64> {
        meval::eval_str(expr).map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to evaluate expression: {}", e))
        })
    }
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculator search provider
pub struct CalculatorProvider {
    /// Expression evaluator
    evaluator: ExpressionEvaluator,
    /// Whether the provider is enabled
    enabled: bool,
    /// Regex for detecting math expressions
    math_pattern: Regex,
}

impl CalculatorProvider {
    /// Creates a new CalculatorProvider
    pub fn new() -> Result<Self> {
        info!("Initializing CalculatorProvider");

        // Pattern to detect potential math expressions
        // Matches expressions with numbers and operators
        let math_pattern = Regex::new(r"^[\d\s\+\-\*/\(\)\.\^%]+$")
            .map_err(|e| LauncherError::ExecutionError(format!("Failed to compile regex: {}", e)))?;

        Ok(Self {
            evaluator: ExpressionEvaluator::new(),
            enabled: true,
            math_pattern,
        })
    }

    /// Checks if a query is a mathematical expression
    fn is_math_expression(&self, query: &str) -> bool {
        let trimmed = query.trim();
        
        // Must not be empty
        if trimmed.is_empty() {
            return false;
        }

        // Must match the pattern
        if !self.math_pattern.is_match(trimmed) {
            return false;
        }

        // Must be a valid expression
        ExpressionEvaluator::is_valid_expression(trimmed)
    }

    /// Formats a number result with appropriate precision
    fn format_result(value: f64) -> String {
        // If the number is an integer, display without decimals
        if value.fract() == 0.0 && value.abs() < 1e15 {
            format!("{}", value as i64)
        } else {
            // Otherwise, display with up to 10 decimal places, removing trailing zeros
            let formatted = format!("{:.10}", value);
            formatted.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }

    /// Converts calculation result to SearchResult
    fn create_search_result(&self, expression: &str, result: f64) -> SearchResult {
        let formatted_result = Self::format_result(result);
        
        let mut metadata = HashMap::new();
        metadata.insert("expression".to_string(), serde_json::json!(expression));
        metadata.insert("result".to_string(), serde_json::json!(result));
        metadata.insert("formatted_result".to_string(), serde_json::json!(formatted_result));

        SearchResult {
            id: format!("calculator:{}", expression),
            title: formatted_result.clone(),
            subtitle: format!("{} = {}", expression, formatted_result),
            icon: Some("calculator".to_string()),
            result_type: ResultType::Calculator,
            score: 100.0, // Always high score for valid calculations
            metadata,
            action: ResultAction::CopyToClipboard {
                content: formatted_result,
            },
        }
    }
}

#[async_trait]
impl SearchProvider for CalculatorProvider {
    fn name(&self) -> &str {
        "Calculator"
    }

    fn priority(&self) -> u8 {
        90 // Very high priority for calculator
    }

    async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let trimmed = query.trim();
        
        if !self.is_math_expression(trimmed) {
            return Ok(Vec::new());
        }

        debug!("Evaluating mathematical expression: '{}'", trimmed);

        // Try to evaluate the expression
        match ExpressionEvaluator::evaluate(trimmed) {
            Ok(result) => {
                debug!("Expression evaluated to: {}", result);
                let search_result = self.create_search_result(trimmed, result);
                Ok(vec![search_result])
            }
            Err(e) => {
                debug!("Failed to evaluate expression: {}", e);
                Ok(Vec::new()) // Return empty results on evaluation error
            }
        }
    }

    async fn execute(&self, result: &SearchResult) -> Result<()> {
        if result.result_type != ResultType::Calculator {
            return Err(LauncherError::ExecutionError(
                "Not a calculator result".to_string(),
            ));
        }

        // Extract the formatted result from metadata
        let formatted_result = result
            .metadata
            .get("formatted_result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                LauncherError::ExecutionError("Invalid calculator result".to_string())
            })?;

        info!("Copying calculator result to clipboard: {}", formatted_result);

        // Copy to clipboard using the action
        match &result.action {
            ResultAction::CopyToClipboard { content } => {
                Self::copy_to_clipboard(content).await?;
                info!("Successfully copied result to clipboard");
                Ok(())
            }
            _ => Err(LauncherError::ExecutionError(
                "Invalid action for calculator result".to_string(),
            )),
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("CalculatorProvider initialized");
        Ok(())
    }
}

impl Default for CalculatorProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            evaluator: ExpressionEvaluator::new(),
            enabled: false,
            math_pattern: Regex::new(r"^[\d\s\+\-\*/\(\)\.\^%]+$").unwrap(),
        })
    }
}

impl CalculatorProvider {
    /// Copies text to the Windows clipboard
    #[cfg(windows)]
    async fn copy_to_clipboard(text: &str) -> Result<()> {
        use windows::Win32::Foundation::*;
        use windows::Win32::System::DataExchange::*;
        use windows::Win32::System::Memory::*;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let text_owned = text.to_string();

        tokio::task::spawn_blocking(move || {
            unsafe {
                // Open the clipboard
                if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
                    return Err(LauncherError::ExecutionError(
                        "Failed to open clipboard".to_string(),
                    ));
                }

                // Empty the clipboard
                if EmptyClipboard().is_err() {
                    CloseClipboard().ok();
                    return Err(LauncherError::ExecutionError(
                        "Failed to empty clipboard".to_string(),
                    ));
                }

                // Convert text to wide string
                let wide: Vec<u16> = OsStr::new(&text_owned)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                // Allocate global memory
                let len = wide.len() * std::mem::size_of::<u16>();
                let hmem = GlobalAlloc(GMEM_MOVEABLE, len)
                    .map_err(|_| LauncherError::ExecutionError("Failed to allocate memory".to_string()))?;

                // Lock the memory and copy the text
                let ptr = GlobalLock(hmem);
                if ptr.is_null() {
                    GlobalFree(hmem).ok();
                    CloseClipboard().ok();
                    return Err(LauncherError::ExecutionError(
                        "Failed to lock memory".to_string(),
                    ));
                }

                std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
                GlobalUnlock(hmem).ok();

                // Set the clipboard data
                const CF_UNICODETEXT: u32 = 13;
                if SetClipboardData(CF_UNICODETEXT, HANDLE(hmem.0)).is_err() {
                    GlobalFree(hmem).ok();
                    CloseClipboard().ok();
                    return Err(LauncherError::ExecutionError(
                        "Failed to set clipboard data".to_string(),
                    ));
                }

                // Close the clipboard
                CloseClipboard().ok();

                Ok(())
            }
        })
        .await
        .map_err(|e| {
            LauncherError::ExecutionError(format!("Failed to spawn clipboard task: {}", e))
        })??;

        Ok(())
    }

    #[cfg(not(windows))]
    async fn copy_to_clipboard(_text: &str) -> Result<()> {
        Err(LauncherError::ExecutionError(
            "Clipboard operations not supported on this platform".to_string(),
        ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculator_provider_creation() {
        let provider = CalculatorProvider::new();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "Calculator");
        assert_eq!(provider.priority(), 90);
        assert!(provider.is_enabled());
    }

    #[tokio::test]
    async fn test_expression_evaluator_validation() {
        // Valid expressions
        assert!(ExpressionEvaluator::is_valid_expression("2+2"));
        assert!(ExpressionEvaluator::is_valid_expression("10 * 5"));
        assert!(ExpressionEvaluator::is_valid_expression("(3 + 4) * 2"));
        assert!(ExpressionEvaluator::is_valid_expression("100 / 4"));
        assert!(ExpressionEvaluator::is_valid_expression("2.5 + 3.7"));
        assert!(ExpressionEvaluator::is_valid_expression("42"));

        // Invalid expressions
        assert!(!ExpressionEvaluator::is_valid_expression("hello"));
        assert!(!ExpressionEvaluator::is_valid_expression("2 + abc"));
        assert!(!ExpressionEvaluator::is_valid_expression(""));
        assert!(!ExpressionEvaluator::is_valid_expression("test 123"));
    }

    #[tokio::test]
    async fn test_expression_evaluation() {
        // Basic arithmetic
        assert_eq!(ExpressionEvaluator::evaluate("2+2").unwrap(), 4.0);
        assert_eq!(ExpressionEvaluator::evaluate("10-5").unwrap(), 5.0);
        assert_eq!(ExpressionEvaluator::evaluate("3*4").unwrap(), 12.0);
        assert_eq!(ExpressionEvaluator::evaluate("20/4").unwrap(), 5.0);

        // With spaces
        assert_eq!(ExpressionEvaluator::evaluate("2 + 2").unwrap(), 4.0);
        assert_eq!(ExpressionEvaluator::evaluate("10 * 5").unwrap(), 50.0);

        // Parentheses and order of operations
        assert_eq!(ExpressionEvaluator::evaluate("(2+3)*4").unwrap(), 20.0);
        assert_eq!(ExpressionEvaluator::evaluate("2+3*4").unwrap(), 14.0);

        // Decimals
        assert_eq!(ExpressionEvaluator::evaluate("2.5+2.5").unwrap(), 5.0);
        assert_eq!(ExpressionEvaluator::evaluate("10.5/2").unwrap(), 5.25);
    }

    #[tokio::test]
    async fn test_is_math_expression() {
        let provider = CalculatorProvider::new().unwrap();

        // Valid math expressions
        assert!(provider.is_math_expression("2+2"));
        assert!(provider.is_math_expression("10 * 5"));
        assert!(provider.is_math_expression("(3+4)*2"));
        assert!(provider.is_math_expression("100/4"));

        // Invalid expressions
        assert!(!provider.is_math_expression("hello"));
        assert!(!provider.is_math_expression("search query"));
        assert!(!provider.is_math_expression(""));
        assert!(!provider.is_math_expression("   "));
    }

    #[tokio::test]
    async fn test_format_result() {
        // Integers
        assert_eq!(CalculatorProvider::format_result(4.0), "4");
        assert_eq!(CalculatorProvider::format_result(100.0), "100");
        assert_eq!(CalculatorProvider::format_result(-5.0), "-5");

        // Decimals
        assert_eq!(CalculatorProvider::format_result(3.14), "3.14");
        assert_eq!(CalculatorProvider::format_result(2.5), "2.5");
        assert_eq!(CalculatorProvider::format_result(10.123456789), "10.123456789");

        // Remove trailing zeros
        assert_eq!(CalculatorProvider::format_result(5.0), "5");
        assert_eq!(CalculatorProvider::format_result(3.10), "3.1");
    }

    #[tokio::test]
    async fn test_search_basic_arithmetic() {
        let provider = CalculatorProvider::new().unwrap();

        // Test addition
        let results = provider.search("2+2").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "4");
        assert_eq!(results[0].result_type, ResultType::Calculator);
        assert_eq!(results[0].score, 100.0);

        // Test multiplication
        let results = provider.search("10*5").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "50");

        // Test division
        let results = provider.search("20/4").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "5");
    }

    #[tokio::test]
    async fn test_search_complex_expressions() {
        let provider = CalculatorProvider::new().unwrap();

        // Test with parentheses
        let results = provider.search("(2+3)*4").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "20");

        // Test order of operations
        let results = provider.search("2+3*4").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "14");

        // Test with decimals
        let results = provider.search("2.5+2.5").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "5");
    }

    #[tokio::test]
    async fn test_search_non_math_query() {
        let provider = CalculatorProvider::new().unwrap();

        // Non-math queries should return empty results
        let results = provider.search("hello world").await.unwrap();
        assert!(results.is_empty());

        let results = provider.search("search query").await.unwrap();
        assert!(results.is_empty());

        let results = provider.search("").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_result_metadata() {
        let provider = CalculatorProvider::new().unwrap();

        let results = provider.search("2+2").await.unwrap();
        assert_eq!(results.len(), 1);

        let result = &results[0];
        
        // Check metadata
        assert!(result.metadata.contains_key("expression"));
        assert!(result.metadata.contains_key("result"));
        assert!(result.metadata.contains_key("formatted_result"));

        let expression = result.metadata.get("expression").unwrap().as_str().unwrap();
        assert_eq!(expression, "2+2");

        let formatted = result.metadata.get("formatted_result").unwrap().as_str().unwrap();
        assert_eq!(formatted, "4");
    }

    #[tokio::test]
    async fn test_search_result_action() {
        let provider = CalculatorProvider::new().unwrap();

        let results = provider.search("2+2").await.unwrap();
        assert_eq!(results.len(), 1);

        let result = &results[0];
        
        // Check action is CopyToClipboard
        match &result.action {
            ResultAction::CopyToClipboard { content } => {
                assert_eq!(content, "4");
            }
            _ => panic!("Expected CopyToClipboard action"),
        }
    }

    #[tokio::test]
    async fn test_execute_invalid_result_type() {
        let provider = CalculatorProvider::new().unwrap();

        // Create a result with wrong type
        let invalid_result = SearchResult {
            id: "test".to_string(),
            title: "Test".to_string(),
            subtitle: "Test".to_string(),
            icon: None,
            result_type: ResultType::File, // Wrong type
            score: 100.0,
            metadata: HashMap::new(),
            action: ResultAction::CopyToClipboard {
                content: "test".to_string(),
            },
        };

        let result = provider.execute(&invalid_result).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_provider_initialization() {
        let mut provider = CalculatorProvider::new().unwrap();
        
        let result = provider.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_with_spaces() {
        let provider = CalculatorProvider::new().unwrap();

        // Test with various spacing
        let results = provider.search("  2 + 2  ").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "4");

        let results = provider.search("10   *   5").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "50");
    }

    #[tokio::test]
    async fn test_invalid_expression_returns_empty() {
        let provider = CalculatorProvider::new().unwrap();

        // Invalid expressions should return empty results, not error
        // Unmatched parentheses - these should fail evaluation
        let results = provider.search("(2+3").await.unwrap();
        assert!(results.is_empty());

        let results = provider.search("2+3)").await.unwrap();
        assert!(results.is_empty());

        // Expression ending with operator
        let results = provider.search("2+").await.unwrap();
        assert!(results.is_empty());
    }
}
