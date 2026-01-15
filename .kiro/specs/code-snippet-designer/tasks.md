# Implementation Plan

- [x] 1. Set up project structure and dependencies
  - Create new Rust project with Cargo.toml
  - Add core dependencies: axum, tokio, serde, image, tesseract-rs, syntect, thiserror, uuid, chrono
  - Set up basic project directory structure (src/handlers, src/services, src/models, src/utils)
  - Configure development environment and basic logging with tracing
  - _Requirements: 5.1, 5.3_

- [x] 2. Implement core data models and error handling
  - [x] 2.1 Create Theme and CodeSnippet data structures
    - Define Theme struct with BackgroundStyle, SyntaxColors, WindowStyle, and TypographyStyle
    - Implement CodeSnippet struct with metadata and serialization
    - Create BackgroundType enum and related styling structures
    - Add validation methods for theme configurations
    - _Requirements: 3.1, 3.2_

  - [x] 2.2 Implement comprehensive error handling system
    - Create AppError enum with specific error types for OCR, image generation, file upload, theme, and language detection
    - Implement ErrorResponse struct with user-friendly messages and actions
    - Add ErrorSeverity enum and ErrorHandler with retry logic
    - Implement retry mechanism with exponential backoff for transient failures
    - _Requirements: 4.5, 5.4_

- [x] 3. Create web server foundation and file handling
  - [x] 3.1 Set up Axum web server with basic routing
    - Configure Axum router with health check endpoint
    - Set up middleware for CORS, logging, request timeout, and static file serving
    - Implement basic server startup with graceful shutdown handling
    - Add environment-based configuration management
    - _Requirements: 5.1, 5.2_

  - [x] 3.2 Implement file upload handling with multipart forms
    - Create multipart form handler for image uploads using tower-multipart
    - Add file validation for supported image formats (PNG, JPG, JPEG)
    - Implement FileStorageService for temporary file storage with UUID-based naming
    - Add file size limits, cleanup routines, and error handling for invalid uploads
    - _Requirements: 1.1, 1.3, 4.5_

- [x] 4. Implement OCR service for screenshot processing
  - [x] 4.1 Create OCR service using tesseract-rs
    - Set up OCRService struct with Tesseract OCR engine initialization
    - Implement text extraction from uploaded images with confidence scoring
    - Add language detection and support for multiple languages
    - Handle OCR processing errors, timeouts, and invalid image formats
    - Create OCRResult struct with text, confidence, and detected language
    - _Requirements: 1.2, 1.4_

  - [x] 4.2 Add OCR result processing and validation
    - Create text cleaning and formatting functions for extracted code
    - Implement confidence threshold validation with user feedback
    - Add manual text editing capability for low-confidence OCR results
    - Handle special characters and code formatting preservation
    - _Requirements: 1.4, 1.5_

- [x] 5. Implement syntax highlighting and language detection
  - [x] 5.1 Set up syntect for code syntax highlighting
    - Configure syntect with SyntaxSet and popular programming language definitions
    - Implement syntax highlighting application with theme integration
    - Create SyntaxHighlighter service with caching for performance
    - Add fallback mechanism for unknown or unsupported languages
    - _Requirements: 2.3, 2.4, 2.5_

  - [x] 5.2 Create language detection service
    - Implement LanguageDetector with heuristic-based detection algorithm
    - Add support for common programming languages (JavaScript, Python, Rust, Go, Java, C++, etc.)
    - Create LanguageResult struct with confidence scoring and alternatives
    - Add manual language selection override capability
    - _Requirements: 2.4, 2.5_

- [x] 6. Implement theme management system
  - [x] 6.1 Create built-in theme definitions and ThemeManager
    - Define default themes (dark, light, high-contrast, popular editor themes like VS Code Dark, Monokai)
    - Implement ThemeManager struct with theme loading, validation, and caching
    - Create theme serialization/deserialization with serde
    - Add theme storage and retrieval mechanisms
    - _Requirements: 3.1, 3.5_

  - [x] 6.2 Add theme customization capabilities
    - Implement color customization for syntax highlighting with SyntaxColors struct
    - Add BackgroundStyle options (solid, gradient, pattern) with BackgroundType enum
    - Create WindowStyle variations (macOS, Windows, terminal, clean) with proper rendering
    - Support TypographyStyle customization (font family, size, line spacing)
    - _Requirements: 3.2, 3.4, 3.5_

- [x] 7. Implement image generation engine
  - [x] 7.1 Create core ImageGenerator with rendering system
    - Set up ImageGenerator struct with image-rs for high-quality image generation
    - Implement code layout calculation and text rendering with proper spacing
    - Integrate syntax highlighting application to rendered text using syntect
    - Create responsive layout system for different code lengths and screen sizes
    - _Requirements: 3.3, 4.1, 4.3_

  - [x] 7.2 Add advanced rendering features
    - Implement window frame rendering with different WindowStyle options
    - Add background rendering supporting solid colors, gradients, and patterns
    - Create padding and margin calculations for proper visual spacing
    - Support for line numbers, code formatting, and typography styling
    - _Requirements: 3.2, 3.4, 3.5_

- [x] 8. Implement export and download functionality
  - [x] 8.1 Create image export service with multiple formats
    - Implement PNG export with configurable quality settings using image-rs
    - Add JPG export with compression options and quality control
    - Support SVG export for vector graphics using resvg
    - Create ExportOptions struct with different resolution options (standard, high, ultra)
    - _Requirements: 4.1, 4.2, 4.3_

  - [x] 8.2 Add download management and file serving
    - Implement file download endpoints with proper HTTP headers and content types
    - Add progress tracking for large image generation with async processing
    - Create temporary file cleanup after download with automatic expiration
    - Handle download failures with retry mechanisms and user feedback
    - _Requirements: 4.4, 4.5_

- [x] 9. Create web frontend interface
  - [x] 9.1 Build responsive HTML structure and CSS styling
    - Create responsive HTML layout for all device sizes with semantic structure
    - Implement CSS styling with mobile-first approach and modern CSS features
    - Add loading indicators, progress feedback, and visual state management
    - Create accessible form elements, navigation, and WCAG 2.1 compliance
    - _Requirements: 5.2, 5.3_

  - [x] 9.2 Implement JavaScript functionality and user interactions
    - Create file upload interface with drag-and-drop support and validation
    - Implement text input area with paste functionality and formatting preservation
    - Add theme selection controls and real-time customization interface
    - Create real-time preview updates and export functionality with progress tracking
    - _Requirements: 1.1, 2.1, 3.2, 4.1, 5.3_

- [x] 10. Add API endpoints and integration
  - [x] 10.1 Create REST API endpoints with proper routing
    - Implement POST /api/upload for image uploads and OCR processing with multipart handling
    - Add POST /api/process for text input processing and syntax highlighting
    - Create GET /api/themes for theme listing and selection with JSON responses
    - Implement POST /api/generate for image generation and download with async processing
    - Add GET /api/health for health checks and monitoring
    - _Requirements: 1.1, 1.2, 2.1, 3.1, 4.1_

  - [x] 10.2 Add session management and caching
    - Implement temporary session storage for user data with UUID-based sessions
    - Add caching for generated images, processed results, and theme data
    - Create cleanup routines for expired sessions and temporary files
    - Add rate limiting and request validation for API endpoints
    - _Requirements: 5.4, 5.5_

- [ ] 11. Testing and validation
  - [ ]* 11.1 Write unit tests for core functionality
    - Test OCR service with various image types and qualities
    - Validate theme management and customization logic
    - Test image generation with different code samples and themes
    - Verify error handling and recovery mechanisms
    - _Requirements: 1.2, 3.1, 4.1, 4.5_

  - [ ]* 11.2 Create integration tests
    - Test complete workflow from upload to download
    - Validate API endpoints with various input scenarios
    - Test frontend-backend integration and error handling
    - Performance testing with large code snippets and high-resolution exports
    - _Requirements: 5.1, 5.4_