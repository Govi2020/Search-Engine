import { motion } from "framer-motion";
import { useState, useRef, useEffect } from "react";
import "./App.css";
import * as math from "mathjs";
import Tetris from "./components/Tetris";

const suggestions = [
  "design systems",
  "Calculator",
  "AI agents",
  "product analytics",
];

function ResultFavicon({ src, alt, fallbackText }) {
  const [hasError, setHasError] = useState(false);

  if (!src || hasError) {
    return <div className="result-favicon-fallback">{fallbackText}</div>;
  }

  return (
    <img
      className="result-favicon"
      src={src}
      alt={alt}
      onError={() => setHasError(true)}
    />
  );
}

function App() {
  const [query, setQuery] = useState("design systems");
  const [mathQuery, setMathQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");

  const [searchSuggestions, setSearchSuggestions] = useState([]);
  const [showSuggestions, setShowSuggestions] = useState(false);

  const [isSearching, setIsSearching] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [visibleResults, setVisibleResults] = useState([]);

  const [featureSnippet, setFeatureSnippet] = useState(null);

  const [copiedUrl, setCopiedUrl] = useState("");
  const [openMenuUrl, setOpenMenuUrl] = useState("");
  const [searchTime, setSearchTime] = useState("");
  const [searchMode, setSearchMode] = useState("web");
  const [previewImage, setPreviewImage] = useState(null);
  const [ipDetails, setIpDetails] = useState(null);
  const [mathResult, setMathResult] = useState(null);
  const [isGameMode, setIsGameMode] = useState(false);

  const abortController = useRef(null);
  const debounceTimer = useRef(null);
  const requestId = useRef(0);

  const getIPInfo = async () => {
    setIpDetails(null);
    try {
      const response = await fetch("http://ip-api.com/json");

      const data = await response.json();

      setIpDetails(data);
      return;
    } catch (error) {
      console.error(error);
    }
  };
  function isParsable(expression) {
    try {
      const node = math.parse(expression);
      return node.type !== "SymbolNode";
    } catch (error) {
      return false;
    }
  }

  const calculateMathResult = () => {
    try {
      setMathResult(math.evaluate(mathQuery));
    } catch (error) {
      setMathResult(0);
    }
  };

  const startSearch = async (nextQuery, mode = searchMode) => {
    const normalized = nextQuery.trim().toLowerCase();


    if (normalized === "") {
      return;
    }

    setQuery(normalized);
    setSubmittedQuery(normalized);
    setSearchMode(mode);
    setIsLoading(true);
    setIsSearching(true);
    setIsGameMode(false);


    const ipRegex = /\b(?:my\s+)?(?:public\s+)?ip(?:\s+address)?\b/i;
    const calculationRegex =
      /^\s*\d+(?:\.\d+)?(?:\s*[+\-*/]\s*\d+(?:\.\d+)?)+\s*$/i;

    setMathResult(null);
    setMathQuery(null);

    if (ipRegex.test(query)) {
      getIPInfo();
    } else if (query.includes("calculator") || isParsable(query)) {
      // Handle calculation
      try {
        setMathResult(math.evaluate(query));
        setMathQuery(query);
      } catch (error) {
        setMathResult(null);
        setMathQuery(null);
      }
    } else if (
      query.toLowerCase().includes("tetris") ||
      query.toLowerCase().includes("play game") ||
      query.toLowerCase().includes("play tetris") ||
      query.toLowerCase().trim() === "game" ||
      query.toLowerCase().trim() === "play"
    ) {
      setIsGameMode(true);
    }

    try {
      const searchEndpoint =
        mode === "images"
          ? "http://localhost:3000/images/"
          : "http://localhost:3000/";

      const [searchResponse, answerResponse] = await Promise.all([
        fetch(`${searchEndpoint}?query=${encodeURIComponent(normalized)}`),
        fetch(
          `http://localhost:3000/answer/?query=${encodeURIComponent(normalized)}`,
        ),
      ]);

      const searchData = await searchResponse.json();

      const timingHeader =
        searchResponse.headers.get("x-search-time-ms") ||
        searchResponse.headers.get("X-Search-Time-MS") ||
        "";

      setVisibleResults(Array.isArray(searchData) ? searchData : []);

      setSearchTime(timingHeader);

      const answerData = await answerResponse.json();

      const isThereAnswer = !(
        answerData.long_answer.trim() == "" &&
        answerData.short_answer.trim() == ""
      );

      if (isThereAnswer && answerData.answer_type !== "unknown") {
        setFeatureSnippet(answerData);
      } else {
        setFeatureSnippet(null);
      }
    } catch (error) {
      console.error(error);

      setVisibleResults([]);
      setFeatureSnippet(null);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (!query.trim()) {
      return;
    }
    clearTimeout(debounceTimer.current);

    debounceTimer.current = setTimeout(() => {
      if (abortController.current) {
        abortController.current.abort();
      }

      // Create new controller
      abortController.current = new AbortController();

      // Increase request number
      const currentRequest = ++requestId.current;

      getSuggestions(currentRequest, abortController.current.signal);
    }, 500);

    // Cleanup when component unmounts
    return () => {
      clearTimeout(debounceTimer.current);
    };
  }, [query]);

  const getSuggestions = async (currentRequest, signal) => {
    const normalized = query.trim();

    if (currentRequest !== requestId.current) {
      return;
    }

    if (!normalized) {
      setSearchSuggestions([]);
      setShowSuggestions(false);
      return;
    }

    try {
      const endpoint = "http://localhost:3000/suggestions/";
      const response = await fetch(
        `${endpoint}?query=${encodeURIComponent(normalized)}`,
        { signal: signal },
      );

      const data = await response.json();

      setSearchSuggestions(Array.isArray(data) ? data.slice(0, 8) : []);
      setShowSuggestions(true);

      if (searchSuggestions.length == 1) {
        if (searchSuggestions[0].trim() == query.trim()) {
          return;
        }
      }
    } catch (error) {
      console.error("Suggestion request failed:", error);
      setSearchSuggestions([]);
    }
  };

  const handleSubmit = (event) => {
    event.preventDefault();
    startSearch(query);
  };

  const getDisplayUrl = (value) => {
    if (!value) return "website";
    return value.replace(/^https?:\/\//i, "").replace(/\/$/, "");
  };

  const getResultHref = (value) => {
    if (!value) return "#";
    const trimmed = value.trim();
    if (/^https?:\/\//i.test(trimmed)) return trimmed;
    return `https://${trimmed}`;
  };

  const getFaviconSource = (item, siteHref = "") => {
    const directSource =
      item?.favicon ||
      item?.faviconUrl ||
      item?.icon ||
      item?.site?.favicon ||
      item?.site?.faviconUrl ||
      item?.site?.icon ||
      "";
    if (directSource) return directSource;

    const candidate = siteHref || item?.site?.url || item?.url || "";
    if (!candidate) return "";

    try {
      const parsed = new URL(
        candidate.includes("://") ? candidate : `https://${candidate}`,
      );
      return `https://www.google.com/s2/favicons?sz=64&domain=${parsed.hostname}`;
    } catch (error) {
      console.error("Unable to build favicon URL:", error);
      return "";
    }
  };

  const getImageDomainLabel = (value) => {
    if (!value) return "website";

    const cleaned = value.replace(/^https?:\/\//i, "").replace(/\/$/, "");
    return cleaned.split("/")[0] || "website";
  };

  const getFallbackText = (value) => {
    if (!value) return "W";
    const cleaned = value.replace(/^https?:\/\//i, "").replace(/\/$/, "");
    const first = cleaned.split(/[./-]/).find(Boolean);
    return (first || "W").slice(0, 1).toUpperCase();
  };

  const getImagePreviewUrl = (item) =>
    item?.imageUrl || item?.src || item?.thumbnail || item?.url || "";

  const getImageSiteHref = (item) => {
    if (!item) return "";

    return (
      item?.site?.url ||
      item?.site?.link ||
      item?.pageUrl ||
      item?.sourceUrl ||
      item?.href ||
      item?.website ||
      item?.url ||
      ""
    );
  };

  const handleSearchChange = (event) => {
    setQuery(event.target.value);
  };

  const openPreview = (item) => {
    const src = getImagePreviewUrl(item);
    if (!src) return;

    setPreviewImage({ src, alt: item?.title || "Image preview" });
  };

  const closePreview = () => {
    setPreviewImage(null);
  };

  const copyLink = async (url) => {
    if (!url) return;

    try {
      await navigator.clipboard.writeText(url);
      setCopiedUrl(url);
      setOpenMenuUrl("");
      window.setTimeout(() => setCopiedUrl(""), 1400);
    } catch (error) {
      console.error("Unable to copy link:", error);
    }
  };

  const toggleResultMenu = (url) => {
    setOpenMenuUrl((current) => (current === url ? "" : url));
  };

  return (
    <div className={`app-shell ${isSearching ? "results-open" : ""}`}>
      <main className="main-stage">
        <section
          className={`hero-panel ${isSearching ? "hero-panel-compact" : ""}`}
        >
          {!isSearching && (
            <motion.div
              className="landing-brand"
              initial={false}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.45, ease: [0.2, 0.8, 0.2, 1] }}
            >
              <div className="brand-badge">G</div>
              <span>Googol</span>
            </motion.div>
          )}

          <motion.form
            className={`search-form ${isSearching ? "search-form-compact" : ""}`}
            onSubmit={handleSubmit}
            initial={false}
            animate={isSearching ? { y: -10, scale: 1 } : { y: 0, scale: 1 }}
            transition={{ duration: 0.65, ease: [0.2, 0.8, 0.2, 1] }}
          >
            <motion.label
              className={`search-input-wrap ${isSearching ? "search-input-wrap-compact" : ""}`}
              htmlFor="search"
              initial={false}
              animate={
                isSearching
                  ? {
                      y: -2,
                      scale: 1.01,
                      boxShadow: "0 16px 44px rgba(0, 0, 0, 0.3)",
                    }
                  : {
                      y: 0,
                      scale: 1,
                      boxShadow: "0 8px 30px rgba(0, 0, 0, 0.2)",
                    }
              }
              transition={{ duration: 0.65, ease: [0.2, 0.8, 0.2, 1] }}
            >
              <motion.div
                className="brand-inline"
                initial={false}
                animate={
                  isSearching
                    ? { opacity: 1, width: 36, marginRight: 8, scale: 1 }
                    : { opacity: 0, width: 0, marginRight: 0, scale: 0.9 }
                }
                transition={{ duration: 0.45, ease: [0.2, 0.8, 0.2, 1] }}
              >
                <div className="brand-badge">G</div>
              </motion.div>
              <span className="search-icon">⌕</span>
              <input
                id="search"
                value={query}
                onChange={handleSearchChange}
                autocomplete="off"
                onFocus={() => {
                  if (searchSuggestions.length)
                    if (searchSuggestions.length == 1) {
                      if (searchSuggestions[0].trim() == query.trim()) {
                        return;
                      }
                    }
                  setShowSuggestions(true);
                }}
                onBlur={() => {
                  setTimeout(() => setShowSuggestions(false), 150);
                }}
                placeholder="Search"
              />
              {showSuggestions && searchSuggestions.length > 0 && (
                <div className="search-suggestions-dropdown">
                  {searchSuggestions.map((suggestion, index) => (
                    <button
                      key={index}
                      className="search-suggestion-item"
                      style={{ background: "transparent" }}
                      type="button"
                      onMouseDown={() => {
                        setQuery(suggestion);
                        setShowSuggestions(false);
                        startSearch(suggestion);
                      }}
                    >
                      <span className="suggestion-icon">⌕</span>

                      <span className="suggestion-text">{suggestion}</span>
                    </button>
                  ))}
                </div>
              )}
            </motion.label>
            <motion.button
              type="submit"
              whileHover={{ scale: 1.02, y: -1 }}
              whileTap={{ scale: 0.98 }}
            >
              Search
            </motion.button>
          </motion.form>

          {!isSearching && (
            <div className="suggestions-row">
              {suggestions.map((suggestion) => (
                <button
                  key={suggestion}
                  className="chip"
                  type="button"
                  onClick={() => startSearch(suggestion)}
                >
                  {suggestion}
                </button>
              ))}
            </div>
          )}
        </section>


        {isSearching && (
          <section className="results-panel">
            <div className="results-header">
              <div className="results-tabs">
                <button
                  className={`results-tab ${searchMode === "web" ? "results-tab-active" : ""}`}
                  type="button"
                  onClick={() => startSearch(query, "web")}
                >
                  Web
                </button>
                <button
                  className={`results-tab ${searchMode === "images" ? "results-tab-active" : ""}`}
                  type="button"
                  onClick={() => startSearch(query, "images")}
                >
                  Images
                </button>
              </div>
              <span className="results-count">
                {isLoading
                  ? "Loading…"
                  : `${visibleResults.length} ${searchMode === "images" ? "images" : "results"}`}
                {searchTime ? ` • in ${searchTime} ms` : ""}
              </span>
            </div>

        {isGameMode && (<><h1 style={{ textAlign: "center" }}>Tetris</h1> <Tetris/></>  )}

            {searchMode === "web" && mathResult !== null && (
              <form
                className="feature-snippet"
                onSubmit={(e) => e.preventDefault()}
              >
                <div className="feature-snippet-label">Calculator</div>

                {mathResult !== null && (
                  <div className="feature-snippet-short">
                    <input
                      type="text"
                      value={mathResult}
                      className="calculator-answer"
                      readOnly
                    />
                  </div>
                )}

                {/* Calcutor Input */}
                <input
                  type="text"
                  value={mathQuery}
                  onChange={(e) => setMathQuery(e.target.value)}
                  className="calculator-input"
                  placeholder="Enter a mathematical expression"
                />

                <div className="calculator-buttons">
                  <button
                    className="chip"
                    type="button"
                    onClick={() => setMathQuery(`${mathQuery}sin(`)}
                  >
                    Sin
                  </button>

                  <button
                    className="chip"
                    type="button"
                    onClick={() => setMathQuery(`${mathQuery}cos(`)}
                  >
                    Cos
                  </button>

                  <button
                    className="chip"
                    type="button"
                    onClick={() => setMathQuery(`${mathQuery}tan(`)}
                  >
                    Tan
                  </button>
                  <button
                    className="chip"
                    type="button"
                    onClick={() => setMathQuery(`${mathQuery}log(`)}
                  >
                    Log
                  </button>

                  <button
                    className="chip"
                    type="button"
                    onClick={() => setMathQuery(`${mathQuery}sqrt(`)}
                  >
                    Sqrt
                  </button>

                  <button
                    className="chip"
                    onClick={() => setMathQuery(`${mathQuery}det(`)}
                    type="button"
                  >
                    Determinant
                  </button>
                </div>

                <button
                  className="chip calculator-calculate-button"
                  type="submit"
                  onClick={() => calculateMathResult()}
                >
                  Calculate
                </button>
              </form>
            )}

            {searchMode === "web" && ipDetails && (
              <section className="feature-snippet">
                <div className="feature-snippet-label">Your IP Info</div>

                {ipDetails.query && (
                  <div className="feature-snippet-short">
                    IP : {ipDetails.query}
                  </div>
                )}

                {
                  <div className="">
                    Location : {ipDetails.city} {ipDetails.regionName},{" "}
                    {ipDetails.country}
                  </div>
                }
              </section>
            )}

            {searchMode === "web" && featureSnippet && (
              <section className="feature-snippet">
                <div className="feature-snippet-label">Answer</div>

                {featureSnippet.short_answer && (
                  <div className="feature-snippet-short">
                    {featureSnippet.short_answer}
                  </div>
                )}

                {featureSnippet.answer_type === "collection" &&
                  featureSnippet.long_answer && (
                    <ol className="feature-snippet-list">
                      {featureSnippet.long_answer.map((item, index) => (
                        <li key={index}>{item}</li>
                      ))}
                    </ol>
                  )}

                {featureSnippet.answer_type !== "collection" &&
                  featureSnippet.long_answer && (
                    <div className="feature-snippet-long">
                      {featureSnippet.long_answer}
                    </div>
                  )}
              </section>
            )}

            <div className="results-list">
              {isLoading ? (
                Array.from({ length: 4 }).map((_, index) => (
                  <div className="skeleton-card" key={`skeleton-${index}`}>
                    <div className="skeleton-line skeleton-line-short" />
                    <div className="skeleton-line skeleton-line-medium" />
                    <div className="skeleton-line skeleton-line-long" />
                  </div>
                ))
              ) : searchMode === "images" ? (
                <div className="image-results-grid">
                  {visibleResults.map((item, index) => {
                    const previewSrc = getImagePreviewUrl(item);
                    const itemHref = getResultHref(
                      getImageSiteHref(item) || previewSrc,
                    );
                    const imageHref = getResultHref(previewSrc);
                    const siteHref = getResultHref(getImageSiteHref(item));
                    const siteName =
                      item?.site?.title ||
                      item?.site?.name ||
                      item?.site?.url ||
                      getDisplayUrl(siteHref);
                    const actionKey = `${imageHref}-${siteHref}-${index}`;
                    const domainLabel = getImageDomainLabel(siteHref);

                    return (
                      <article
                        className="image-result-item"
                        key={`${previewSrc}-${index}`}
                      >
                        <button
                          className="image-result-link"
                          type="button"
                          onClick={() => openPreview(item)}
                          aria-label={`Preview ${item.title || "image result"}`}
                        >
                          <img
                            className="image-result-image"
                            src={previewSrc}
                            alt={item.title || "Search result image"}
                          />
                        </button>

                        <div className="image-result-meta">
                          <div className="image-result-meta-row">
                            <a
                              className="image-result-site"
                              href={siteHref}
                              target="_blank"
                              rel="noreferrer"
                            >
                              {siteName && (
                                <span className="image-result-title">
                                  {siteName}
                                </span>
                              )}
                              <span className="image-result-site-badge">
                                <ResultFavicon
                                  src={getFaviconSource(item, siteHref)}
                                  alt={`${siteName} favicon`}
                                  fallbackText={getFallbackText(
                                    siteHref || previewSrc,
                                  )}
                                />
                                <span className="image-result-domain">
                                  {domainLabel}
                                </span>
                              </span>
                            </a>

                            <div className="result-action-wrapper">
                              <button
                                className="result-action"
                                type="button"
                                onClick={(event) => {
                                  event.stopPropagation();
                                  toggleResultMenu(actionKey);
                                }}
                                aria-label="Open image actions"
                              >
                                {copiedUrl === imageHref ||
                                copiedUrl === siteHref
                                  ? "✓"
                                  : "⋯"}
                              </button>

                              {openMenuUrl === actionKey && (
                                <div className="result-action-menu" role="menu">
                                  <button
                                    className="result-action-menu-item"
                                    type="button"
                                    onClick={(event) => {
                                      event.stopPropagation();
                                      copyLink(imageHref);
                                    }}
                                    role="menuitem"
                                  >
                                    Copy image link
                                  </button>
                                  <button
                                    className="result-action-menu-item"
                                    type="button"
                                    onClick={(event) => {
                                      event.stopPropagation();
                                      copyLink(siteHref);
                                    }}
                                    role="menuitem"
                                  >
                                    Copy website link
                                  </button>
                                </div>
                              )}
                            </div>
                          </div>
                        </div>
                      </article>
                    );
                  })}
                </div>
              ) : (
                visibleResults.map((item) => {
                  const itemHref = getResultHref(item.url);

                  return (
                    <article
                      className="result-item"
                      key={`${item.title}-${item.url}`}
                    >
                      <div className="result-favicon-wrap">
                        <ResultFavicon
                          src={getFaviconSource(item, itemHref)}
                          alt={`${item.title} favicon`}
                          fallbackText={getFallbackText(item.url)}
                        />
                      </div>

                      <div className="result-content">
                        <a
                          className="result-url"
                          href={itemHref}
                          target="_blank"
                          rel="noreferrer"
                        >
                          {getDisplayUrl(item.url)}
                        </a>
                        <a
                          className="result-title-link"
                          href={itemHref}
                          target="_blank"
                          rel="noreferrer"
                        >
                          {item.title}
                        </a>
                        <p className="result-description">{item.description}</p>
                      </div>

                      <div className="result-action-wrapper">
                        <button
                          className="result-action"
                          type="button"
                          onClick={(event) => {
                            event.stopPropagation();
                            toggleResultMenu(itemHref);
                          }}
                          aria-label="Open result actions"
                        >
                          {copiedUrl === itemHref ? "✓" : "⋯"}
                        </button>

                        {openMenuUrl === itemHref && (
                          <div className="result-action-menu" role="menu">
                            <button
                              className="result-action-menu-item"
                              type="button"
                              onClick={(event) => {
                                event.stopPropagation();
                                copyLink(itemHref);
                              }}
                              role="menuitem"
                            >
                              Copy link
                            </button>
                          </div>
                        )}
                      </div>
                    </article>
                  );
                })
              )}
            </div>
          </section>
        )}
      </main>

      {previewImage && (
        <div
          className="image-preview-overlay"
          role="dialog"
          aria-modal="true"
          onClick={closePreview}
        >
          <div
            className="image-preview-card"
            onClick={(event) => event.stopPropagation()}
          >
            <button
              className="image-preview-close"
              type="button"
              onClick={closePreview}
              aria-label="Close preview"
            >
              ×
            </button>
            <img
              className="image-preview-image"
              src={previewImage.src}
              alt={previewImage.alt}
            />
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
