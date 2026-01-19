// Code Snippet Designer - Frontend JavaScript
// Handles all user interactions and API communication

class CodeSnippetDesigner {
  constructor() {
    this.currentCode = "";
    this.currentLanguage = "auto";
    this.currentTheme = "default";
    this.currentStep = "input";
    this.downloadId = null;
    this.progressInterval = null;

    this.init();
  }

  init() {
    this.setupEventListeners();
    this.setupDragAndDrop();
    this.loadThemes();
    this.updateProgress();
  }

  setupEventListeners() {
    // Tab switching
    document.querySelectorAll(".tab-button").forEach((button) => {
      button.addEventListener("click", (e) => this.switchTab(e));
    });

    // Navigation
    document.querySelectorAll(".nav-link").forEach((link) => {
      link.addEventListener("click", (e) => this.smoothScroll(e));
    });

    // Mobile navigation toggle
    const navToggle = document.querySelector(".nav-toggle");
    const navMenu = document.querySelector(".nav-menu");
    if (navToggle && navMenu) {
      navToggle.addEventListener("click", () => {
        const isExpanded = navToggle.getAttribute("aria-expanded") === "true";
        navToggle.setAttribute("aria-expanded", !isExpanded);
        navMenu.classList.toggle("active");
      });
    }

    // File upload
    const fileInput = document.getElementById("file-input");
    if (fileInput) {
      fileInput.addEventListener("change", (e) => this.handleFileUpload(e));
    }

    // Text input areas
    const textareas = ["paste-textarea", "type-textarea", "ocr-text"];
    textareas.forEach((id) => {
      const textarea = document.getElementById(id);
      if (textarea) {
        textarea.addEventListener("input", (e) => this.handleTextInput(e));
        textarea.addEventListener("paste", (e) => this.handlePaste(e));
      }
    });

    // Language selection
    const languageSelect = document.getElementById("language-select");
    if (languageSelect) {
      languageSelect.addEventListener("change", (e) =>
        this.handleLanguageChange(e),
      );
    }

    // Theme and styling controls
    this.setupStylingControls();

    // Export controls
    this.setupExportControls();

    // OCR actions
    const retryOcrBtn = document.getElementById("retry-ocr");
    const useOcrBtn = document.getElementById("use-ocr-text");
    if (retryOcrBtn)
      retryOcrBtn.addEventListener("click", () => this.retryOCR());
    if (useOcrBtn) useOcrBtn.addEventListener("click", () => this.useOCRText());

    // Preview refresh
    const refreshBtn = document.getElementById("refresh-preview");
    if (refreshBtn)
      refreshBtn.addEventListener("click", () => this.refreshPreview());

    // Toast close buttons
    document.querySelectorAll(".toast-close").forEach((button) => {
      button.addEventListener("click", (e) => this.closeToast(e));
    });

    // Keyboard shortcuts
    document.addEventListener("keydown", (e) =>
      this.handleKeyboardShortcuts(e),
    );
  }

  setupDragAndDrop() {
    const uploadArea = document.getElementById("upload-area");
    if (!uploadArea) return;

    ["dragenter", "dragover", "dragleave", "drop"].forEach((eventName) => {
      uploadArea.addEventListener(eventName, this.preventDefaults, false);
    });

    ["dragenter", "dragover"].forEach((eventName) => {
      uploadArea.addEventListener(
        eventName,
        () => {
          uploadArea.classList.add("dragover");
        },
        false,
      );
    });

    ["dragleave", "drop"].forEach((eventName) => {
      uploadArea.addEventListener(
        eventName,
        () => {
          uploadArea.classList.remove("dragover");
        },
        false,
      );
    });

    uploadArea.addEventListener("drop", (e) => this.handleDrop(e), false);
  }

  setupStylingControls() {
    // Background type change
    const bgType = document.getElementById("background-type");
    if (bgType) {
      bgType.addEventListener("change", (e) =>
        this.handleBackgroundTypeChange(e),
      );
    }

    // Color inputs
    document.querySelectorAll(".color-input").forEach((input) => {
      input.addEventListener("change", () => this.updatePreview());
    });

    // Range inputs
    document.querySelectorAll(".range-input").forEach((input) => {
      input.addEventListener("input", (e) => this.handleRangeInput(e));
    });

    // Style selects
    document.querySelectorAll(".style-select").forEach((select) => {
      select.addEventListener("change", () => this.updatePreview());
    });
  }

  setupExportControls() {
    const generateBtn = document.getElementById("generate-btn");
    const downloadBtn = document.getElementById("download-btn");
    const exportFormat = document.getElementById("export-format");

    if (generateBtn) {
      generateBtn.addEventListener("click", () => this.generateImage());
    }

    if (downloadBtn) {
      downloadBtn.addEventListener("click", () => this.downloadImage());
    }

    if (exportFormat) {
      exportFormat.addEventListener("change", (e) =>
        this.handleFormatChange(e),
      );
    }
  }

  preventDefaults(e) {
    e.preventDefault();
    e.stopPropagation();
  }

  switchTab(e) {
    const clickedTab = e.currentTarget;
    const targetPanel = clickedTab.getAttribute("aria-controls");

    // Update tab buttons
    document.querySelectorAll(".tab-button").forEach((tab) => {
      tab.classList.remove("active");
      tab.setAttribute("aria-selected", "false");
    });
    clickedTab.classList.add("active");
    clickedTab.setAttribute("aria-selected", "true");

    // Update tab panels
    document.querySelectorAll(".tab-panel").forEach((panel) => {
      panel.classList.remove("active");
    });
    document.getElementById(targetPanel).classList.add("active");

    // Show language selection for text inputs
    const languageSelection = document.getElementById("language-selection");
    if (targetPanel !== "upload-panel") {
      languageSelection.style.display = "block";
    } else {
      languageSelection.style.display = "none";
    }
  }

  smoothScroll(e) {
    e.preventDefault();
    const targetId = e.currentTarget.getAttribute("href");
    const targetElement = document.querySelector(targetId);
    if (targetElement) {
      targetElement.scrollIntoView({
        behavior: "smooth",
        block: "start",
      });
    }
  }

  async handleFileUpload(e) {
    const file = e.target.files[0];
    if (!file) return;

    if (!this.validateFile(file)) return;

    this.showLoading(
      "Processing image...",
      "Extracting text from your image using OCR",
    );

    try {
      const formData = new FormData();
      // Use field name 'image' (server accepts 'image' or 'file')
      formData.append("image", file);

      const response = await fetch("/api/upload", {
        method: "POST",
        body: formData,
      });

      const serverResult = await response.json();

      if (response.ok) {
        // serverResult has shape: { success, message, files: [ { file_id, filename, size, content_type, extension, ocr: { ... } } ] }
        if (serverResult.files && serverResult.files.length > 0) {
          const fileInfo = serverResult.files[0];
          // Provide file metadata feedback in the UI
          const ocrTitle = document.querySelector(".ocr-title");
          if (ocrTitle) {
            const name = fileInfo.filename || "Uploaded Image";
            const size = fileInfo.size
              ? `${Math.round(fileInfo.size / 1024)} KB`
              : "";
            ocrTitle.textContent = `Extracted Text — ${name} ${size ? `(${size})` : ""}`;
          }

          this.displayOCRResults(serverResult);
        } else {
          // No files array returned — fall back to old behavior
          this.displayOCRResults(serverResult);
        }

        this.showSuccess("Image processed successfully!");
      } else {
        // If server returned JSON with helpful message, display it
        const msg =
          serverResult && serverResult.message
            ? serverResult.message
            : "Failed to process image";
        throw new Error(msg);
      }
    } catch (error) {
      console.error("Upload error:", error);
      this.showError(
        "Failed to process image: " +
          (error && error.message ? error.message : error),
      );
    } finally {
      this.hideLoading();
    }
  }

  handleDrop(e) {
    const files = e.dataTransfer.files;
    if (files.length > 0) {
      const fileInput = document.getElementById("file-input");
      fileInput.files = files;
      this.handleFileUpload({ target: { files } });
    }
  }

  validateFile(file) {
    const maxSize = 10 * 1024 * 1024; // 10MB
    const allowedTypes = ["image/png", "image/jpg", "image/jpeg"];

    if (!allowedTypes.includes(file.type)) {
      this.showError("Please upload a PNG, JPG, or JPEG image.");
      return false;
    }

    if (file.size > maxSize) {
      this.showError("File size must be less than 10MB.");
      return false;
    }

    return true;
  }

  displayOCRResults(result) {
    // Accept either:
    // - old shape: { text, confidence, language, ... }
    // - new shape: { success, message, files: [ { ..., ocr: { success, text, confidence, detected_language, needs_review, error, help } } ] }
    const ocrResults = document.getElementById("ocr-results");
    const ocrText = document.getElementById("ocr-text");
    const ocrConfidence = document.getElementById("ocr-confidence");

    // Normalize to an 'ocr' object and 'fileInfo' for metadata
    let ocr = null;
    let fileInfo = null;

    if (result && result.files && result.files.length > 0) {
      fileInfo = result.files[0];
      ocr = fileInfo.ocr || null;
    } else if (
      result &&
      (result.text || result.confidence || result.language)
    ) {
      // Backwards compatible: result is the OCR object itself
      ocr = {
        success: true,
        text: result.text,
        confidence: result.confidence || 0,
        detected_language: result.language,
      };
    } else {
      // Nothing useful
      ocr = null;
    }

    if (!ocrResults || !ocrText || !ocrConfidence) return;

    if (!ocr) {
      // No OCR info — show message to user
      ocrText.value = "";
      ocrConfidence.className = "ocr-confidence low";
      ocrConfidence.textContent = "No OCR data available";
      ocrResults.style.display = "block";
      return;
    }

    // If the OCR operation failed on the server, surface the message/help
    if (ocr.success === false) {
      ocrText.value = ocr.text || "";
      ocrConfidence.className = "ocr-confidence low";
      // Prefer explicit error text, otherwise a generic message
      if (ocr.error || ocr.message) {
        const errMsg = ocr.message || ocr.error || "OCR failed";
        ocrConfidence.textContent = `OCR unavailable: ${errMsg}`;
      } else if (ocr.help) {
        ocrConfidence.textContent = `OCR unavailable. ${ocr.help}`;
      } else {
        ocrConfidence.textContent = "OCR unavailable";
      }

      // If server provided a 'help' suggestion (e.g., rebuild instructions), surface it
      if (ocr.help) {
        const helpNode = document.createElement("div");
        helpNode.className = "ocr-help";
        helpNode.textContent = ocr.help;
        if (!document.getElementById("ocr-help")) {
          helpNode.id = "ocr-help";
          ocrResults.appendChild(helpNode);
        } else {
          document.getElementById("ocr-help").textContent = ocr.help;
        }
      }

      ocrResults.style.display = "block";
      return;
    }

    // Successful OCR result
    const confidence = typeof ocr.confidence === "number" ? ocr.confidence : 0;
    let confidenceClass = "low";
    let confidenceText = "Low confidence";

    if (confidence > 0.8) {
      confidenceClass = "high";
      confidenceText = "High confidence";
    } else if (confidence > 0.6) {
      confidenceClass = "medium";
      confidenceText = "Medium confidence";
    }

    ocrText.value = ocr.text || "";
    ocrConfidence.className = `ocr-confidence ${confidenceClass}`;
    ocrConfidence.textContent = `${confidenceText} (${Math.round(confidence * 100)}%)`;

    // Show detected language if present
    const detectedLang = ocr.detected_language || ocr.language || null;
    if (detectedLang) {
      this.currentLanguage = detectedLang;
      const languageSelect = document.getElementById("language-select");
      if (
        languageSelect &&
        Array.from(languageSelect.options).some(
          (opt) => opt.value === detectedLang,
        )
      ) {
        languageSelect.value = detectedLang;
      }
    }

    // Display the OCR results panel
    ocrResults.style.display = "block";

    // Optionally show additional file metadata near the OCR panel
    if (fileInfo) {
      let metaNode = document.getElementById("ocr-file-meta");
      if (!metaNode) {
        metaNode = document.createElement("div");
        metaNode.id = "ocr-file-meta";
        metaNode.className = "ocr-file-meta";
        const ocrContainer = document.getElementById("ocr-results");
        ocrContainer.insertBefore(metaNode, ocrContainer.firstChild);
      }
      metaNode.textContent = `${fileInfo.filename || "file"} — ${fileInfo.extension ? fileInfo.extension.toUpperCase() : ""} ${fileInfo.size ? `• ${Math.round(fileInfo.size / 1024)} KB` : ""}`;
    }
  }

  retryOCR() {
    const fileInput = document.getElementById("file-input");
    if (fileInput.files.length > 0) {
      this.handleFileUpload({ target: { files: fileInput.files } });
    }
  }

  useOCRText() {
    const ocrText = document.getElementById("ocr-text");
    if (ocrText) {
      this.currentCode = ocrText.value;
      this.updateCodeStats();
      this.updatePreview();
      this.setStep("customize");
    }
  }

  handleTextInput(e) {
    this.currentCode = e.target.value;
    this.updateCodeStats();
    this.detectLanguage();
    this.updatePreview();

    if (this.currentCode.trim()) {
      this.setStep("customize");
    }
  }

  handlePaste(e) {
    // Allow default paste behavior, then process
    setTimeout(() => {
      this.handleTextInput(e);
    }, 10);
  }

  handleLanguageChange(e) {
    this.currentLanguage = e.target.value;
    this.updatePreview();
  }

  detectLanguage() {
    if (this.currentLanguage !== "auto") return;

    // Simple language detection based on keywords and syntax
    const code = this.currentCode.toLowerCase();

    const patterns = {
      javascript: [
        /function\s+\w+/,
        /const\s+\w+/,
        /let\s+\w+/,
        /var\s+\w+/,
        /=>\s*{?/,
        /console\.log/,
      ],
      typescript: [
        /interface\s+\w+/,
        /type\s+\w+/,
        /:\s*string/,
        /:\s*number/,
        /:\s*boolean/,
      ],
      python: [
        /def\s+\w+/,
        /import\s+\w+/,
        /from\s+\w+\s+import/,
        /if\s+__name__\s*==/,
        /print\(/,
      ],
      rust: [
        /fn\s+\w+/,
        /let\s+mut/,
        /struct\s+\w+/,
        /impl\s+\w+/,
        /use\s+std::/,
      ],
      go: [
        /func\s+\w+/,
        /package\s+\w+/,
        /import\s+\(/,
        /fmt\.Print/,
        /var\s+\w+\s+\w+/,
      ],
      java: [
        /public\s+class/,
        /public\s+static\s+void\s+main/,
        /System\.out\.print/,
        /private\s+\w+/,
      ],
      cpp: [/#include\s*</, /std::/, /cout\s*<</, /int\s+main\s*\(/],
      html: [/<html/, /<div/, /<span/, /<p>/, /<h[1-6]>/],
      css: [/\{\s*\w+\s*:/, /\.[\w-]+\s*\{/, /#[\w-]+\s*\{/, /@media/],
      json: [/^\s*\{/, /"\w+"\s*:/, /\[\s*\{/],
    };

    for (const [lang, regexes] of Object.entries(patterns)) {
      const matches = regexes.filter((regex) => regex.test(code)).length;
      if (matches >= 2) {
        this.currentLanguage = lang;
        const languageSelect = document.getElementById("language-select");
        if (languageSelect) {
          languageSelect.value = lang;
        }
        break;
      }
    }
  }

  updateCodeStats() {
    const lines = this.currentCode.split("\n").length;
    const chars = this.currentCode.length;

    const lineCount = document.getElementById("line-count");
    const charCount = document.getElementById("char-count");

    if (lineCount) lineCount.textContent = lines;
    if (charCount) charCount.textContent = chars;
  }

  async loadThemes() {
    try {
      // For now, create some default themes
      const themes = [
        {
          id: "default",
          name: "Default Dark",
          background: { type: "solid", primary: "#1e1e1e" },
          preview: "linear-gradient(135deg, #1e1e1e 0%, #2d2d2d 100%)",
        },
        {
          id: "light",
          name: "Light",
          background: { type: "solid", primary: "#ffffff" },
          preview: "linear-gradient(135deg, #ffffff 0%, #f8f9fa 100%)",
        },
        {
          id: "monokai",
          name: "Monokai",
          background: { type: "solid", primary: "#272822" },
          preview: "linear-gradient(135deg, #272822 0%, #3e3d32 100%)",
        },
        {
          id: "dracula",
          name: "Dracula",
          background: { type: "solid", primary: "#282a36" },
          preview: "linear-gradient(135deg, #282a36 0%, #44475a 100%)",
        },
        {
          id: "github",
          name: "GitHub",
          background: { type: "solid", primary: "#f6f8fa" },
          preview: "linear-gradient(135deg, #f6f8fa 0%, #ffffff 100%)",
        },
        {
          id: "gradient-blue",
          name: "Blue Gradient",
          background: {
            type: "gradient",
            primary: "#667eea",
            secondary: "#764ba2",
          },
          preview: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
        },
      ];

      this.renderThemes(themes);
    } catch (error) {
      console.error("Failed to load themes:", error);
    }
  }

  renderThemes(themes) {
    const themeGrid = document.getElementById("theme-grid");
    if (!themeGrid) return;

    themeGrid.innerHTML = themes
      .map(
        (theme) => `
            <div class="theme-card" data-theme-id="${theme.id}" role="button" tabindex="0" aria-label="Select ${theme.name} theme">
                <div class="theme-preview" style="background: ${theme.preview}"></div>
                <div class="theme-name">${theme.name}</div>
            </div>
        `,
      )
      .join("");

    // Add click handlers
    themeGrid.querySelectorAll(".theme-card").forEach((card) => {
      card.addEventListener("click", () =>
        this.selectTheme(card.dataset.themeId),
      );
      card.addEventListener("keydown", (e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          this.selectTheme(card.dataset.themeId);
        }
      });
    });

    // Select default theme
    this.selectTheme("default");
  }

  selectTheme(themeId) {
    // Update UI
    document.querySelectorAll(".theme-card").forEach((card) => {
      card.classList.remove("active");
    });
    document
      .querySelector(`[data-theme-id="${themeId}"]`)
      .classList.add("active");

    this.currentTheme = themeId;
    this.updatePreview();
  }

  handleBackgroundTypeChange(e) {
    const bgType = e.target.value;
    const bgColor2 = document.getElementById("bg-color-2");

    if (bgType === "gradient") {
      bgColor2.style.display = "block";
    } else {
      bgColor2.style.display = "none";
    }

    this.updatePreview();
  }

  handleRangeInput(e) {
    const input = e.target;
    const valueDisplay = document.getElementById(input.id + "-value");

    if (valueDisplay) {
      let value = input.value;
      if (input.id === "font-size") {
        value += "px";
      } else if (input.id === "padding") {
        value += "px";
      } else if (input.id === "export-quality") {
        value += "%";
      }
      valueDisplay.textContent = value;
    }

    this.updatePreview();
  }

  handleFormatChange(e) {
    const format = e.target.value;
    const qualityGroup = document.getElementById("quality-group");

    if (format === "jpg") {
      qualityGroup.style.display = "block";
    } else {
      qualityGroup.style.display = "none";
    }
  }

  updatePreview() {
    if (!this.currentCode.trim()) return;

    const previewArea = document.getElementById("preview-area");
    if (!previewArea) return;

    // Create a simple preview (in a real implementation, this would render the actual styled code)
    const preview = document.createElement("div");
    preview.style.cssText = `
            background: ${this.getBackgroundStyle()};
            color: ${this.getTextColor()};
            font-family: ${document.getElementById("font-family")?.value || "JetBrains Mono"}, monospace;
            font-size: ${document.getElementById("font-size")?.value || 14}px;
            padding: ${document.getElementById("padding")?.value || 32}px;
            border-radius: 8px;
            white-space: pre-wrap;
            overflow: auto;
            max-height: 300px;
            width: 100%;
            box-sizing: border-box;
        `;

    preview.textContent = this.currentCode;

    previewArea.innerHTML = "";
    previewArea.appendChild(preview);
  }

  getBackgroundStyle() {
    const bgType = document.getElementById("background-type")?.value || "solid";
    const bgColor1 = document.getElementById("bg-color-1")?.value || "#1e1e1e";
    const bgColor2 = document.getElementById("bg-color-2")?.value || "#2d2d2d";

    if (bgType === "gradient") {
      return `linear-gradient(135deg, ${bgColor1} 0%, ${bgColor2} 100%)`;
    }
    return bgColor1;
  }

  getTextColor() {
    const bgColor = document.getElementById("bg-color-1")?.value || "#1e1e1e";
    // Simple contrast calculation
    const hex = bgColor.replace("#", "");
    const r = parseInt(hex.substr(0, 2), 16);
    const g = parseInt(hex.substr(2, 2), 16);
    const b = parseInt(hex.substr(4, 2), 16);
    const brightness = (r * 299 + g * 587 + b * 114) / 1000;
    return brightness > 128 ? "#000000" : "#ffffff";
  }

  refreshPreview() {
    this.updatePreview();
    this.showSuccess("Preview refreshed!");
  }

  async generateImage() {
    if (!this.currentCode.trim()) {
      this.showError("Please enter some code first.");
      return;
    }

    const generateBtn = document.getElementById("generate-btn");
    const progressContainer = document.getElementById("generation-progress");

    generateBtn.disabled = true;
    progressContainer.style.display = "block";

    try {
      const requestData = {
        code: this.currentCode,
        language:
          this.currentLanguage === "auto" ? "text" : this.currentLanguage,
        theme: this.currentTheme,
        options: {
          format: document.getElementById("export-format")?.value || "png",
          resolution:
            document.getElementById("export-resolution")?.value || "1x",
          quality: document.getElementById("export-quality")?.value || 90,
          background_type:
            document.getElementById("background-type")?.value || "solid",
          background_color_1:
            document.getElementById("bg-color-1")?.value || "#1e1e1e",
          background_color_2:
            document.getElementById("bg-color-2")?.value || "#2d2d2d",
          window_style:
            document.getElementById("window-style")?.value || "macos",
          font_family:
            document.getElementById("font-family")?.value || "JetBrains Mono",
          font_size: parseInt(
            document.getElementById("font-size")?.value || 14,
          ),
          padding: parseInt(document.getElementById("padding")?.value || 32),
        },
      };

      const response = await fetch("/api/generate", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(requestData),
      });

      const result = await response.json();

      if (response.ok) {
        this.downloadId = result.download_id;
        this.startProgressTracking();
      } else {
        throw new Error(result.message || "Failed to generate image");
      }
    } catch (error) {
      console.error("Generation error:", error);
      this.showError("Failed to generate image: " + error.message);
      this.resetGenerationUI();
    }
  }

  startProgressTracking() {
    if (!this.downloadId) return;

    this.progressInterval = setInterval(async () => {
      try {
        const response = await fetch(
          `/api/generate/progress/${this.downloadId}`,
        );
        const result = await response.json();

        if (response.ok) {
          this.updateGenerationProgress(result);

          if (result.status === "completed") {
            this.onGenerationComplete();
          } else if (result.status === "failed") {
            throw new Error(result.message || "Generation failed");
          }
        } else {
          throw new Error("Failed to check progress");
        }
      } catch (error) {
        console.error("Progress check error:", error);
        this.showError("Failed to check generation progress");
        this.resetGenerationUI();
      }
    }, 1000);
  }

  updateGenerationProgress(result) {
    const progressFill = document.getElementById("generation-progress-fill");
    const progressText = document.getElementById("generation-progress-text");

    if (progressFill) {
      progressFill.style.width = `${result.progress || 0}%`;
    }

    if (progressText) {
      progressText.textContent = result.message || "Generating image...";
    }
  }

  onGenerationComplete() {
    if (this.progressInterval) {
      clearInterval(this.progressInterval);
      this.progressInterval = null;
    }

    const generateBtn = document.getElementById("generate-btn");
    const downloadBtn = document.getElementById("download-btn");
    const progressContainer = document.getElementById("generation-progress");

    if (generateBtn) generateBtn.disabled = false;
    if (downloadBtn) downloadBtn.style.display = "inline-flex";
    if (progressContainer) progressContainer.style.display = "none";

    // Show visible download link and inline preview
    const downloadLink = document.getElementById("download-link");
    const generatedImage = document.getElementById("generated-image");
    const generatedImageContainer = document.getElementById(
      "generated-image-container",
    );

    if (this.downloadId && downloadLink) {
      const downloadUrl = `/api/generate/download/${this.downloadId}`;
      downloadLink.href = downloadUrl;
      const ext =
        (document.getElementById("export-format") &&
          document.getElementById("export-format").value) ||
        "png";
      downloadLink.setAttribute("download", `code-snippet.${ext}`);
      downloadLink.style.display = "inline-flex";
    }

    // Fetch a preview image and show it inline (non-blocking)
    if (this.downloadId && generatedImage) {
      const previewUrl = `/api/generate/download/${this.downloadId}`;
      fetch(previewUrl, { method: "GET", cache: "no-store" })
        .then((resp) => {
          if (!resp.ok) throw new Error(`Preview fetch failed: ${resp.status}`);
          return resp.blob();
        })
        .then((blob) => {
          const url = URL.createObjectURL(blob);
          generatedImage.src = url;
          if (generatedImageContainer)
            generatedImageContainer.style.display = "block";
          // Revoke after a short delay to allow viewing
          setTimeout(() => URL.revokeObjectURL(url), 10000);
        })
        .catch((err) => {
          console.warn("Failed to load generated image preview:", err);
          if (generatedImageContainer)
            generatedImageContainer.style.display = "none";
        });
    }

    this.showSuccess(
      "Image generated successfully! Use the Download link to save it.",
    );
    this.setStep("export");
  }

  resetGenerationUI() {
    if (this.progressInterval) {
      clearInterval(this.progressInterval);
      this.progressInterval = null;
    }

    const generateBtn = document.getElementById("generate-btn");
    const progressContainer = document.getElementById("generation-progress");

    generateBtn.disabled = false;
    progressContainer.style.display = "none";
  }

  async downloadImage() {
    if (!this.downloadId) {
      this.showError("No image available for download.");
      return;
    }

    const downloadUrl = `/api/generate/download/${this.downloadId}`;

    // First attempt: direct navigation / anchor href download.
    // This lets the browser handle the GET and honor Content-Disposition.
    try {
      // Try to determine a sensible filename from the selected export format
      const ext = document.getElementById("export-format")?.value || "png";
      let filename = `code-snippet.${ext}`;

      // Create a temporary anchor and click it. This usually triggers a download
      // without needing to fetch the blob client-side.
      const anchor = document.createElement("a");
      anchor.href = downloadUrl;
      anchor.setAttribute("download", filename);
      anchor.style.display = "none";
      // Important: append to DOM for some browsers to allow programmatic click
      document.body.appendChild(anchor);

      // Use a click to trigger the download; if browser blocks it (popup/blocker),
      // we'll fall back to fetch+blob below.
      anchor.click();

      // Clean up the anchor
      document.body.removeChild(anchor);

      // Give some time for browser to start the download. We still show success to the user.
      this.showSuccess(
        "Download started (if your browser allows automatic downloads).",
      );
      return;
    } catch (err) {
      // If direct navigation fails, fall back to fetch+blob approach below.
      console.warn(
        "Direct navigation download failed, falling back to fetch+blob:",
        err,
      );
    }

    // Fallback: fetch the file as a blob and trigger a programmatic download.
    try {
      const response = await fetch(downloadUrl);

      if (!response.ok) {
        // Try to extract JSON or text error message and surface it
        let errMsg = `Download failed (status ${response.status})`;
        try {
          const json = await response.json();
          if (json && json.message) {
            errMsg = json.message;
          } else {
            errMsg = JSON.stringify(json);
          }
        } catch (_) {
          try {
            const txt = await response.text();
            if (txt) errMsg = txt;
          } catch (_) {
            // ignore
          }
        }
        throw new Error(errMsg);
      }

      const contentType = (
        response.headers.get("content-type") || ""
      ).toLowerCase();
      if (!contentType.startsWith("image/")) {
        // Unexpected content type — read text for diagnostics
        let bodyText = "";
        try {
          bodyText = await response.text();
        } catch (_) {}
        throw new Error(
          `Unexpected content type: ${contentType} ${bodyText ? "- " + bodyText : ""}`,
        );
      }

      const blob = await response.blob();

      // Determine filename from Content-Disposition header if present
      let filename = `code-snippet.${document.getElementById("export-format")?.value || "png"}`;
      const cd = response.headers.get("content-disposition");
      if (cd) {
        const match = /filename\*?=(?:UTF-8'')?"?([^\";]+)"?/.exec(cd);
        if (match && match[1]) {
          try {
            filename = decodeURIComponent(match[1]);
          } catch (_) {
            filename = match[1];
          }
        }
      }

      // Trigger download via anchor with object URL
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.style.display = "none";
      a.href = url;
      a.setAttribute("download", filename);
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      // Revoke after a short delay to ensure the browser has started the download
      setTimeout(() => window.URL.revokeObjectURL(url), 1000);

      this.showSuccess("Image downloaded successfully!");
    } catch (error) {
      console.error("Download error (fallback):", error);
      const msg = error && error.message ? error.message : String(error);
      this.showError("Failed to download image: " + msg);
    }
  }

  setStep(step) {
    this.currentStep = step;
    this.updateProgress();
  }

  updateProgress() {
    const steps = ["input", "customize", "export"];
    const currentIndex = steps.indexOf(this.currentStep);

    document.querySelectorAll(".progress-step").forEach((step, index) => {
      step.classList.remove("active", "completed");

      if (index < currentIndex) {
        step.classList.add("completed");
      } else if (index === currentIndex) {
        step.classList.add("active");
      }
    });

    const progressFill = document.querySelector(
      ".progress-container .progress-fill",
    );
    if (progressFill) {
      const progress = ((currentIndex + 1) / steps.length) * 100;
      progressFill.style.width = `${progress}%`;
    }
  }

  handleKeyboardShortcuts(e) {
    // Ctrl/Cmd + Enter to generate
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      this.generateImage();
    }

    // Escape to close modals/toasts
    if (e.key === "Escape") {
      this.hideLoading();
      this.closeAllToasts();
    }
  }

  showLoading(title = "Loading...", message = "Please wait...") {
    const overlay = document.getElementById("loading-overlay");
    const titleEl = document.getElementById("loading-title");
    const textEl = document.getElementById("loading-text");

    if (titleEl) titleEl.textContent = title;
    if (textEl) textEl.textContent = message;
    if (overlay) overlay.style.display = "flex";
  }

  hideLoading() {
    const overlay = document.getElementById("loading-overlay");
    if (overlay) overlay.style.display = "none";
  }

  showError(message) {
    this.showToast("error", message);
  }

  showSuccess(message) {
    this.showToast("success", message);
  }

  showToast(type, message) {
    const toast = document.getElementById(`${type}-toast`);
    const messageEl = document.getElementById(`${type}-message`);

    if (toast && messageEl) {
      messageEl.textContent = message;
      toast.style.display = "block";

      // Trigger animation
      setTimeout(() => {
        toast.classList.add("show");
      }, 10);

      // Auto-hide after 5 seconds
      setTimeout(() => {
        this.hideToast(toast);
      }, 5000);
    }
  }

  closeToast(e) {
    const toast = e.target.closest(".toast");
    this.hideToast(toast);
  }

  hideToast(toast) {
    if (toast) {
      toast.classList.remove("show");
      setTimeout(() => {
        toast.style.display = "none";
      }, 300);
    }
  }

  closeAllToasts() {
    document.querySelectorAll(".toast").forEach((toast) => {
      this.hideToast(toast);
    });
  }
}

// Initialize the application when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
  new CodeSnippetDesigner();
});

// Service Worker registration for offline support (optional)
if ("serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    navigator.serviceWorker
      .register("/static/sw.js")
      .then((registration) => {
        console.log("SW registered: ", registration);
      })
      .catch((registrationError) => {
        console.log("SW registration failed: ", registrationError);
      });
  });
}
