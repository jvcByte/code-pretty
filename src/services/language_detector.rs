use crate::models::errors::AppError;
use crate::services::syntax_highlighter::SyntaxHighlighter;
use std::collections::HashMap;

/// Result of language detection
#[derive(Debug, Clone)]
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

    /// Pattern-based language detection as fallback
    fn detect_by_patterns(&self, code: &str) -> LanguageResult {
        let mut scores: HashMap<String, f32> = HashMap::new();
        
        for (language, patterns) in &self.language_patterns {
            let mut score = 0.0;
            
            for pattern in patterns {
                if code.contains(pattern) {
                    score += 1.0;
                }
            }
            
            if score > 0.0 {
                // Normalize score by number of patterns
                scores.insert(language.clone(), score / patterns.len() as f32);
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
        let alternatives: Vec<String> = sorted_scores
            .iter()
            .skip(1)
            .take(3)
            .map(|(lang, _)| lang.clone())
            .collect();

        LanguageResult {
            language: best_match.0.clone(),
            confidence: best_match.1.min(0.8), // Cap confidence for pattern-based detection
            alternatives,
        }
    }

    /// Builds language detection patterns
    fn build_language_patterns() -> HashMap<String, Vec<&'static str>> {
        let mut patterns = HashMap::new();

        patterns.insert("Rust".to_string(), vec![
            "fn ", "let ", "mut ", "impl ", "struct ", "enum ", "trait ",
            "use ", "mod ", "pub ", "match ", "if let", "while let",
            "Result<", "Option<", "Vec<", "&str", "String::", "println!",
        ]);

        patterns.insert("JavaScript".to_string(), vec![
            "function ", "const ", "let ", "var ", "=&gt; ", "console.log",
            "document.", "window.", "require(", "import ", "export ",
            "async ", "await ", "Promise", ".then(", ".catch(",
        ]);

        patterns.insert("TypeScript".to_string(), vec![
            "interface ", "type ", ": string", ": number", ": boolean",
            "function ", "const ", "let ", "import ", "export ",
            "async ", "await ", "Promise<", "Array<", "Record<",
        ]);

        patterns.insert("Python".to_string(), vec![
            "def ", "class ", "import ", "from ", "if __name__",
            "print(", "len(", "range(", "for ", "while ", "elif ",
            "try:", "except:", "finally:", "with ", "lambda ",
        ]);

        patterns.insert("Java".to_string(), vec![
            "public class", "private ", "protected ", "public static",
            "import java.", "System.out", "String ", "int ", "void ",
            "new ", "extends ", "implements ", "interface ", "@Override",
        ]);

        patterns.insert("C++".to_string(), vec![
            "#include", "using namespace", "std::", "cout <<", "cin >>",
            "int main", "class ", "struct ", "template<", "vector<",
            "string ", "void ", "const ", "auto ", "nullptr",
        ]);

        patterns.insert("C".to_string(), vec![
            "#include", "int main", "printf(", "scanf(", "malloc(",
            "free(", "struct ", "typedef ", "void ", "char ",
            "FILE *", "NULL", "sizeof(", "return 0;",
        ]);

        patterns.insert("Go".to_string(), vec![
            "package ", "import ", "func ", "var ", "const ",
            "type ", "struct {", "interface {", "go ", "defer ",
            "make(", "len(", "cap(", "range ", "chan ",
        ]);

        patterns.insert("C#".to_string(), vec![
            "using System", "namespace ", "public class", "private ",
            "public static", "Console.", "string ", "int ", "void ",
            "new ", "class ", "interface ", "struct ", "[Attribute]",
        ]);

        patterns.insert("PHP".to_string(), vec![
            "&lt;?php", "$", "echo ", "print ", "function ", "class ",
            "public ", "private ", "protected ", "require ", "include ",
            "array(", "foreach ", "isset(", "empty(", "die(",
        ]);

        patterns.insert("Ruby".to_string(), vec![
            "def ", "class ", "module ", "require ", "include ",
            "puts ", "print ", "p ", "end", "do |", "each do",
            "attr_", "initialize", "self.", "@", "||=",
        ]);

        patterns.insert("Swift".to_string(), vec![
            "func ", "var ", "let ", "class ", "struct ", "enum ",
            "import ", "protocol ", "extension ", "if let", "guard let",
            "print(", "String", "Int", "Bool", "Array<", "Dictionary<",
        ]);

        patterns.insert("Kotlin".to_string(), vec![
            "fun ", "val ", "var ", "class ", "object ", "interface ",
            "import ", "package ", "when ", "is ", "as ", "in ",
            "println(", "String", "Int", "Boolean", "List<", "Map<",
        ]);

        patterns.insert("HTML".to_string(), vec![
            "&lt;html", "&lt;head", "&lt;body", "&lt;div", "&lt;span", "&lt;p",
            "&lt;a href", "&lt;img", "&lt;script", "&lt;style", "&lt;link",
            "class=", "id=", "&lt;!DOCTYPE", "&lt;meta", "&lt;title",
        ]);

        patterns.insert("CSS".to_string(), vec![
            "{", "}", ":", ";", "color:", "background:", "margin:",
            "padding:", "font-", "border:", "width:", "height:",
            ".class", "#id", "@media", "px", "em", "rem",
        ]);

        patterns.insert("SQL".to_string(), vec![
            "SELECT ", "FROM ", "WHERE ", "INSERT ", "UPDATE ",
            "DELETE ", "CREATE ", "ALTER ", "DROP ", "JOIN ",
            "GROUP BY", "ORDER BY", "HAVING ", "UNION ", "INDEX",
        ]);

        patterns.insert("Shell".to_string(), vec![
            "#!/bin/", "echo ", "cd ", "ls ", "grep ", "awk ",
            "sed ", "cat ", "chmod ", "chown ", "sudo ", "export ",
            "if [", "then", "fi", "for ", "while ", "case ",
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
}