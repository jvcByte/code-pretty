# Integration Tests for Code Snippet Designer

This directory contains comprehensive integration tests for the Code Snippet Designer application.

## Test Suites

### 1. Integration Tests (`integration_tests.rs`)

Tests the complete workflow and API endpoints:

- **test_complete_upload_to_download_workflow**: Tests the full user journey from processing code to generating and downloading images
- **test_api_endpoints_with_various_inputs**: Validates API endpoints with different input scenarios (empty code, various languages, manual language selection)
- **test_theme_endpoints**: Tests theme listing, retrieval, and customization endpoints
- **test_file_upload_scenarios**: Tests file upload error handling (missing boundary, empty uploads)
- **test_error_handling**: Validates error responses for invalid JSON, missing fields, and non-existent resources
- **test_health_check**: Verifies health check endpoints are responding correctly

### 2. Performance Tests (`performance_tests.rs`)

Tests system performance and scalability:

- **test_large_code_snippet_processing**: Processes 1000-function code snippet and verifies completion within 5 seconds
- **test_high_resolution_export**: Tests high-resolution image generation (2000x1500) and measures response time
- **test_concurrent_requests**: Validates handling of 10 concurrent requests
- **test_multiple_large_snippets**: Tests memory management with 5 sequential large code snippets
- **test_response_time_consistency**: Measures response time consistency across 20 identical requests
- **test_api_throughput**: Tests API throughput with 50 concurrent requests

### 3. Frontend-Backend Integration Tests (`frontend_backend_tests.rs`)

Tests frontend-backend integration scenarios:

- **test_static_file_serving**: Validates static file serving
- **test_complete_user_workflow**: Simulates complete user interaction flow
- **test_theme_customization_workflow**: Tests theme customization process
- **test_error_recovery_workflow**: Validates error handling and recovery
- **test_multiple_language_processing**: Tests language detection for multiple programming languages
- **test_export_format_options**: Validates export format options endpoint
- **test_session_persistence**: Tests session management across multiple operations

## Running Tests

### Run all integration tests:
```bash
cargo test --test integration_tests
```

### Run performance tests:
```bash
cargo test --test performance_tests
```

### Run frontend-backend tests:
```bash
cargo test --test frontend_backend_tests
```

### Run all tests:
```bash
cargo test
```

### Run specific test:
```bash
cargo test --test integration_tests test_health_check
```

### Run with output:
```bash
cargo test --test integration_tests -- --nocapture
```

## Test Coverage

The integration tests cover:

- ✅ Complete workflow from code input to image download
- ✅ API endpoint validation with various input scenarios
- ✅ Error handling and recovery mechanisms
- ✅ Theme management and customization
- ✅ Language detection for multiple programming languages
- ✅ Performance with large code snippets (1000+ lines)
- ✅ High-resolution image generation
- ✅ Concurrent request handling
- ✅ Response time consistency
- ✅ API throughput and scalability

## Requirements Validated

These tests validate the following requirements from the specification:

- **Requirement 1.1-1.5**: File upload and OCR processing
- **Requirement 2.1-2.5**: Text input and language detection
- **Requirement 3.1-3.5**: Theme management and customization
- **Requirement 4.1-4.5**: Image generation and download
- **Requirement 5.1-5.5**: Performance and responsiveness

## Notes

- Some tests may skip image generation if system dependencies (fonts, image libraries) are not available
- Performance tests measure actual execution time and validate against requirements
- Tests use temporary directories for file storage that are cleaned up automatically
- All tests are designed to be idempotent and can run in parallel
