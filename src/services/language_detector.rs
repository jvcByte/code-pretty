use crate::models::errors::AppError;
use crate::services::syntax_highlighter::SyntaxHighlighter;
use std::collections::HashMap;

/// Result of language detection
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LanguageResult {
    pub language: String,
    pub confidence: f32,
    pub alternatives: Vec<String>,
}

/// Service for detecting programming languages from code content
pub struct LanguageDetector {
    syntax_highlighter: SyntaxHighlighter,
    language_patterns: HashMap<String, Vec<&'static str>>,
}

impl LanguageDetector {
    /// Creates a new LanguageDetector
    pub fn new() -> Result<Self, AppError> {
        let syntax_highlighter = SyntaxHighlighter::new()?;
        let language_patterns = Self::build_language_patterns();
        
        Ok(LanguageDetector {
            syntax_highlighter,
            language_patterns,
        })
    }

    /// Detects the programming language from code content
    pub fn detect_language(&self, code: &str) -> LanguageResult {
        // First try syntect's built-in detection
        if let Some(language) = self.syntax_highlighter.detect_language_from_content(code) {
            return LanguageResult {
                language: language.clone(),
                confidence: 0.9,
                alternatives: vec![],
            };
        }

        // Fallback to pattern-based detection
        self.detect_by_patterns(code)
    }

    /// Detects language from file extension
    pub fn detect_from_extension(&self, extension: &str) -> Option<LanguageResult> {
        self.syntax_highlighter
            .detect_language_from_extension(extension)
            .map(|language| LanguageResult {
                language,
                confidence: 0.95,
                alternatives: vec![],
            })
    }

    /// Gets a list of supported languages
    pub fn get_supported_languages(&self) -> Vec<String> {
        self.syntax_highlighter.get_supported_languages()
    }

    /// Gets a list of supported file extensions
    pub fn get_supported_extensions(&self) -> Vec<String> {
        self.syntax_highlighter.get_supported_extensions()
    }

    /// Validates if a language is supported and returns a LanguageResult for manual selection
    pub fn validate_manual_selection(&self, language: &str) -> Result<LanguageResult, AppError> {
        let supported_languages = self.get_supported_languages();
        
        // Check if the language is supported (case-insensitive)
        let normalized_language = language.trim();
        let found_language = supported_languages
            .iter()
            .find(|&lang| lang.to_lowercase() == normalized_language.to_lowercase())
            .cloned();

        match found_language {
            Some(lang) => Ok(LanguageResult {
                language: lang,
                confidence: 1.0, // Manual selection has highest confidence
                alternatives: vec![],
            }),
            None => {
                // Find similar languages for suggestions
                let alternatives = self.find_similar_languages(normalized_language, &supported_languages);
                
                Err(AppError::LanguageDetectionError {
                    message: format!(
                        "Language '{}' is not supported. Did you mean one of: {}?", 
                        language,
                        alternatives.join(", ")
                    )
                })
            }
        }
    }

    /// Creates a LanguageResult for manual override (bypasses validation)
    pub fn create_manual_override(&self, language: &str) -> LanguageResult {
        LanguageResult {
            language: language.to_string(),
            confidence: 1.0, // Manual selection has highest confidence
            alternatives: vec![],
        }
    }

    /// Finds similar language names for suggestions
    fn find_similar_languages(&self, target: &str, supported: &[String]) -> Vec<String> {
        let target_lower = target.to_lowercase();
        let mut suggestions = Vec::new();

        // Find languages that contain the target as substring
        for lang in supported {
            let lang_lower = lang.to_lowercase();
            if lang_lower.contains(&target_lower) || target_lower.contains(&lang_lower) {
                suggestions.push(lang.clone());
            }
        }

        // If no substring matches, find languages with similar starting letters
        if suggestions.is_empty() {
            for lang in supported {
                let lang_lower = lang.to_lowercase();
                if !target_lower.is_empty() && !lang_lower.is_empty() {
                    if lang_lower.starts_with(&target_lower[..1]) {
                        suggestions.push(lang.clone());
                    }
                }
            }
        }

        // Limit to 3 suggestions
        suggestions.truncate(3);
        suggestions
    }

    /// Pattern-based language detection as fallback
    fn detect_by_patterns(&self, code: &str) -> LanguageResult {
        let mut scores: HashMap<String, f32> = HashMap::new();
        
        for (language, patterns) in &self.language_patterns {
            let mut score = 0.0;
            let mut pattern_matches = 0;
            
            for pattern in patterns {
                if code.contains(pattern) {
                    score += 1.0;
                    pattern_matches += 1;
                }
            }
            
            if pattern_matches > 0 {
                // Weight score by both number of matches and pattern strength
                let pattern_strength = pattern_matches as f32 / patterns.len() as f32;
                let weighted_score = score * pattern_strength;
                scores.insert(language.clone(), weighted_score);
            }
        }

        if scores.is_empty() {
            return LanguageResult {
                language: "Plain Text".to_string(),
                confidence: 0.1,
                alternatives: vec![],
            };
        }

        // Sort by score and get the best match
        let mut sorted_scores: Vec<_> = scores.into_iter().collect();
        sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let best_match = &sorted_scores[0];
        
        // Only return a language if it has a reasonable confidence
        if best_match.1 < 0.3 {
            return LanguageResult {
                language: "Plain Text".to_string(),
                confidence: 0.1,
                alternatives: vec![],
            };
        }

        let alternatives: Vec<String> = sorted_scores
            .iter()
            .skip(1)
            .take(3)
            .map(|(lang, _)| lang.clone())
            .collect();

        LanguageResult {
            language: best_match.0.clone(),
            confidence: (best_match.1 / 5.0).min(0.8), // Normalize and cap confidence
            alternatives,
        }
    }

    /// Builds language detection patterns
    fn build_language_patterns() -> HashMap<String, Vec<&'static str>> {
        let mut patterns = HashMap::new();

        patterns.insert("Rust".to_string(), vec![
            "fn main()", "fn ", "let mut ", "impl ", "struct ", "enum ", "trait ",
            "use ", "mod ", "pub ", "match ", "if let", "while let",
            "Result<", "Option<", "Vec<", "&str", "String::", "println!(",
            "cargo", ".unwrap()", ".expect(",
        ]);

        patterns.insert("JavaScript".to_string(), vec![
            "function(", "function ", "const ", "let ", "var ", "=> ", "console.log(",
            "document.", "window.", "require(", "import ", "export ",
            "async ", "await ", "Promise", ".then(", ".catch(",
            "JSON.", "Array.", "Object.",
        ]);

        patterns.insert("TypeScript".to_string(), vec![
            "interface ", "type ", ": string", ": number", ": boolean",
            "function ", "const ", "let ", "import ", "export ",
            "async ", "await ", "Promise<", "Array<", "Record<",
            "as ", "extends ", "implements ",
        ]);

        patterns.insert("Python".to_string(), vec![
            "def ", "class ", "import ", "from ", "if __name__ == '__main__':",
            "print(", "len(", "range(", "for ", "while ", "elif ",
            "try:", "except:", "finally:", "with ", "lambda ",
            "self.", "__init__", "pass", "None",
        ]);

        patterns.insert("Java".to_string(), vec![
            "public class ", "private ", "protected ", "public static void main",
            "import java.", "System.out.", "String ", "int ", "void ",
            "new ", "extends ", "implements ", "interface ", "@Override",
            "public ", "static ", "final ",
        ]);

        patterns.insert("C++".to_string(), vec![
            "#include <", "using namespace std", "std::", "cout <<", "cin >>",
            "int main(", "class ", "struct ", "template<", "vector<",
            "string ", "void ", "const ", "auto ", "nullptr",
            "::", "->", "delete ", "new ",
        ]);

        patterns.insert("C".to_string(), vec![
            "#include <", "int main(", "printf(", "scanf(", "malloc(",
            "free(", "struct ", "typedef ", "void ", "char ",
            "FILE *", "NULL", "sizeof(", "return 0;",
            "#define", "static ", "extern ",
        ]);

        patterns.insert("Go".to_string(), vec![
            "package main", "package ", "import ", "func main()", "func ", "var ", "const ",
            "type ", "struct {", "interface {", "go ", "defer ",
            "make(", "len(", "cap(", "range ", "chan ",
            "fmt.", "err != nil",
        ]);

        patterns.insert("C#".to_string(), vec![
            "using System", "namespace ", "public class ", "private ",
            "public static void Main", "Console.", "string ", "int ", "void ",
            "new ", "class ", "interface ", "struct ", "[Attribute]",
            "public ", "private ", "protected ",
        ]);

        patterns.insert("PHP".to_string(), vec![
            "<?php", "$_", "echo ", "print ", "function ", "class ",
            "public ", "private ", "protected ", "require ", "include ",
            "array(", "foreach ", "isset(", "empty(", "die(",
            "->", "::", "namespace ",
        ]);

        patterns.insert("Ruby".to_string(), vec![
            "def ", "class ", "module ", "require ", "include ",
            "puts ", "print ", "p ", "end\n", "do |", ".each do",
            "attr_", "initialize", "self.", "@", "||=",
            "rescue ", "ensure ", "yield",
        ]);

        patterns.insert("Swift".to_string(), vec![
            "func ", "var ", "let ", "class ", "struct ", "enum ",
            "import ", "protocol ", "extension ", "if let", "guard let",
            "print(", "String", "Int", "Bool", "Array<", "Dictionary<",
            "override ", "init(", "deinit",
        ]);

        patterns.insert("Kotlin".to_string(), vec![
            "fun main(", "fun ", "val ", "var ", "class ", "object ", "interface ",
            "import ", "package ", "when ", "is ", "as ", "in ",
            "println(", "String", "Int", "Boolean", "List<", "Map<",
            "override ", "companion object",
        ]);

        patterns.insert("HTML".to_string(), vec![
            "<!DOCTYPE html>", "<html", "<head>", "<body>", "<div", "<span", "<p>",
            "<a href", "<img", "<script", "<style", "<link",
            "class=", "id=", "<meta", "<title>",
            "</html>", "</body>", "</head>",
        ]);

        patterns.insert("CSS".to_string(), vec![
            " {", "}", ": ", "; ", "color:", "background:", "margin:",
            "padding:", "font-", "border:", "width:", "height:",
            ".class", "#id", "@media", "px;", "em;", "rem;",
            "display:", "position:", "flex",
        ]);

        patterns.insert("SQL".to_string(), vec![
            "SELECT ", "FROM ", "WHERE ", "INSERT INTO", "UPDATE ",
            "DELETE FROM", "CREATE TABLE", "ALTER TABLE", "DROP ", "JOIN ",
            "GROUP BY", "ORDER BY", "HAVING ", "UNION ", "INDEX",
            "PRIMARY KEY", "FOREIGN KEY", "NOT NULL",
        ]);

        patterns.insert("Shell".to_string(), vec![
            "#!/bin/bash", "#!/bin/sh", "echo ", "cd ", "ls ", "grep ", "awk ",
            "sed ", "cat ", "chmod ", "chown ", "sudo ", "export ",
            "if [ ", "then", "fi", "for ", "while ", "case ",
            "$1", "$@", "$(", "${",
        ]);

        patterns
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create default LanguageDetector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detector_creation() {
        let detector = LanguageDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_rust_detection() {
        let detector = LanguageDetector::new().unwrap();
        let code = r#"
fn main() {
    let x = 5;
    println!("Hello, world!");
}
"#;
        let result = detector.detect_language(code);
        assert_eq!(result.language, "Rust");
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_javascript_detection() {
        let detector = LanguageDetector::new().unwrap();
        let code = r#"
function hello() {
    console.log("Hello, world!");
    const x = 5;
}
"#;
        let result = detector.detect_language(code);
        assert_eq!(result.language, "JavaScript");
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_extension_detection() {
        let detector = LanguageDetector::new().unwrap();
        
        let result = detector.detect_from_extension("rs");
        assert!(result.is_some());
        assert_eq!(result.unwrap().language, "Rust");
        
        let result = detector.detect_from_extension("js");
        assert!(result.is_some());
        assert_eq!(result.unwrap().language, "JavaScript");
    }

    #[test]
    fn test_unknown_code_fallback() {
        let detector = LanguageDetector::new().unwrap();
        let code = "This is just plain text with no programming patterns";
        let result = detector.detect_language(code);
        assert_eq!(result.language, "Plain Text");
        assert!(result.confidence < 0.5);
    }

    #[test]
    fn test_supported_languages() {
        let detector = LanguageDetector::new().unwrap();
        let languages = detector.get_supported_languages();
        assert!(!languages.is_empty());
        assert!(languages.contains(&"Rust".to_string()));
    }

    #[test]
    fn test_manual_language_selection_valid() {
        let detector = LanguageDetector::new().unwrap();
        
        // Test valid language selection
        let result = detector.validate_manual_selection("Rust");
        assert!(result.is_ok());
        let lang_result = result.unwrap();
        assert_eq!(lang_result.language, "Rust");
        assert_eq!(lang_result.confidence, 1.0);
        
        // Test case-insensitive selection
        let result = detector.validate_manual_selection("rust");
        assert!(result.is_ok());
        let lang_result = result.unwrap();
        assert_eq!(lang_result.language, "Rust");
    }

    #[test]
    fn test_manual_language_selection_invalid() {
        let detector = LanguageDetector::new().unwrap();
        
        // Test invalid language selection
        let result = detector.validate_manual_selection("InvalidLanguage");
        assert!(result.is_err());
        
        // Test that error message contains suggestions
        if let Err(AppError::LanguageDetectionError { message }) = result {
            assert!(message.contains("not supported"));
            assert!(message.contains("Did you mean"));
        }
    }

    #[test]
    fn test_manual_override() {
        let detector = LanguageDetector::new().unwrap();
        
        // Test manual override (bypasses validation)
        let result = detector.create_manual_override("CustomLanguage");
        assert_eq!(result.language, "CustomLanguage");
        assert_eq!(result.confidence, 1.0);
        assert!(result.alternatives.is_empty());
    }

    #[test]
    fn test_similar_language_suggestions() {
        let detector = LanguageDetector::new().unwrap();
        
        // Test partial match suggestions
        let result = detector.validate_manual_selection("Java");
        if result.is_ok() {
            // If Java is supported, test with a partial match
            let result = detector.validate_manual_selection("Jav");
            if let Err(AppError::LanguageDetectionError { message }) = result {
                assert!(message.contains("Java"));
            }
        }
    }
}