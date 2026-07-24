import { motion } from 'framer-motion'
import { useState } from 'react'
import './App.css'

const suggestions = ['design systems', 'semantic search', 'AI agents', 'product analytics']

function ResultFavicon({ src, alt, fallbackText }) {
  const [hasError, setHasError] = useState(false)

  if (!src || hasError) {
    return <div className="result-favicon-fallback">{fallbackText}</div>
  }

  return <img className="result-favicon" src={src} alt={alt} onError={() => setHasError(true)} />
}

function App() {
  const [query, setQuery] = useState('design systems')
  const [submittedQuery, setSubmittedQuery] = useState('')
  const [isSearching, setIsSearching] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [visibleResults, setVisibleResults] = useState([])
  const [copiedUrl, setCopiedUrl] = useState('')
  const [openMenuUrl, setOpenMenuUrl] = useState('')
  const [searchTime, setSearchTime] = useState('')
  const [searchMode, setSearchMode] = useState('web')
  const [previewImage, setPreviewImage] = useState(null)

  const startSearch = async (nextQuery, mode = searchMode) => {
    const normalized = nextQuery.trim() || 'design systems'
    setQuery(normalized)
    setSubmittedQuery(normalized)
    setSearchMode(mode)
    setIsLoading(true)
    setIsSearching(true)

    try {
      const endpoint = mode === 'images' ? 'http://localhost:3000/images/' : 'http://localhost:3000/'
      const response = await fetch(`${endpoint}?query=${encodeURIComponent(normalized)}`)
      const data = await response.json()
      const timingHeader = response.headers.get('x-search-time-ms') || response.headers.get('X-Search-Time-MS') || ''
      setSearchTime(timingHeader)
      setVisibleResults(Array.isArray(data) ? data : [])
    } catch (error) {
      console.error('Search request failed:', error)
      setVisibleResults([])
    } finally {
      setIsLoading(false)
    }
  }

  const handleSubmit = (event) => {
    event.preventDefault()
    startSearch(query)
  }

  const getDisplayUrl = (value) => {
    if (!value) return 'website'
    return value.replace(/^https?:\/\//i, '').replace(/\/$/, '')
  }

  const getResultHref = (value) => {
    if (!value) return '#'
    const trimmed = value.trim()
    if (/^https?:\/\//i.test(trimmed)) return trimmed
    return `https://${trimmed}`
  }

  const getFaviconSource = (item, siteHref = '') => {
    const directSource = item?.favicon || item?.faviconUrl || item?.icon || item?.site?.favicon || item?.site?.faviconUrl || item?.site?.icon || ''
    if (directSource) return directSource

    const candidate = siteHref || item?.site?.url || item?.url || ''
    if (!candidate) return ''

    try {
      const parsed = new URL(candidate.includes('://') ? candidate : `https://${candidate}`)
      return `https://www.google.com/s2/favicons?sz=64&domain=${parsed.hostname}`
    } catch (error) {
      console.error('Unable to build favicon URL:', error)
      return ''
    }
  }

  const getImageDomainLabel = (value) => {
    if (!value) return 'website'

    const cleaned = value.replace(/^https?:\/\//i, '').replace(/\/$/, '')
    return cleaned.split('/')[0] || 'website'
  }

  const getFallbackText = (value) => {
    if (!value) return 'W'
    const cleaned = value.replace(/^https?:\/\//i, '').replace(/\/$/, '')
    const first = cleaned.split(/[./-]/).find(Boolean)
    return (first || 'W').slice(0, 1).toUpperCase()
  }

  const getImagePreviewUrl = (item) => item?.imageUrl || item?.src || item?.thumbnail || item?.url || ''

  const getImageSiteHref = (item) => {
    if (!item) return ''

    return (
      item?.site?.url ||
      item?.site?.link ||
      item?.pageUrl ||
      item?.sourceUrl ||
      item?.href ||
      item?.website ||
      item?.url ||
      ''
    )
  }

  const openPreview = (item) => {
    const src = getImagePreviewUrl(item)
    if (!src) return

    setPreviewImage({ src, alt: item?.title || 'Image preview' })
  }

  const closePreview = () => {
    setPreviewImage(null)
  }

  const copyLink = async (url) => {
    if (!url) return

    try {
      await navigator.clipboard.writeText(url)
      setCopiedUrl(url)
      setOpenMenuUrl('')
      window.setTimeout(() => setCopiedUrl(''), 1400)
    } catch (error) {
      console.error('Unable to copy link:', error)
    }
  }

  const toggleResultMenu = (url) => {
    setOpenMenuUrl((current) => (current === url ? '' : url))
  }

  return (
    <div className={`app-shell ${isSearching ? 'results-open' : ''}`}>
      <main className="main-stage">
        <section className={`hero-panel ${isSearching ? 'hero-panel-compact' : ''}`}>
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
            className={`search-form ${isSearching ? 'search-form-compact' : ''}`}
            onSubmit={handleSubmit}
            initial={false}
            animate={isSearching ? { y: -10, scale: 1 } : { y: 0, scale: 1 }}
            transition={{ duration: 0.65, ease: [0.2, 0.8, 0.2, 1] }}
          >
            <motion.label
              className={`search-input-wrap ${isSearching ? 'search-input-wrap-compact' : ''}`}
              htmlFor="search"
              initial={false}
              animate={isSearching ? { y: -2, scale: 1.01, boxShadow: '0 16px 44px rgba(0, 0, 0, 0.3)' } : { y: 0, scale: 1, boxShadow: '0 8px 30px rgba(0, 0, 0, 0.2)' }}
              transition={{ duration: 0.65, ease: [0.2, 0.8, 0.2, 1] }}
            >
              <motion.div
                className="brand-inline"
                initial={false}
                animate={isSearching ? { opacity: 1, width: 36, marginRight: 8, scale: 1 } : { opacity: 0, width: 0, marginRight: 0, scale: 0.9 }}
                transition={{ duration: 0.45, ease: [0.2, 0.8, 0.2, 1] }}
              >
                <div className="brand-badge">G</div>
              </motion.div>
              <span className="search-icon">⌕</span>
              <input
                id="search"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search"
              />
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
                  className={`results-tab ${searchMode === 'web' ? 'results-tab-active' : ''}`}
                  type="button"
                  onClick={() => startSearch(query, 'web')}
                >
                  Web
                </button>
                <button
                  className={`results-tab ${searchMode === 'images' ? 'results-tab-active' : ''}`}
                  type="button"
                  onClick={() => startSearch(query, 'images')}
                >
                  Images
                </button>
              </div>
              <span className="results-count">
                {isLoading ? 'Loading…' : `${visibleResults.length} ${searchMode === 'images' ? 'images' : 'results'}`}
                {searchTime ? ` • in ${searchTime} ms` : ''}
              </span>
            </div>

            <div className="results-list">
              {isLoading ? (
                Array.from({ length: 4 }).map((_, index) => (
                  <div className="skeleton-card" key={`skeleton-${index}`}>
                    <div className="skeleton-line skeleton-line-short" />
                    <div className="skeleton-line skeleton-line-medium" />
                    <div className="skeleton-line skeleton-line-long" />
                  </div>
                ))
              ) : searchMode === 'images' ? (
                <div className="image-results-grid">
                  {visibleResults.map((item, index) => {
                    const previewSrc = getImagePreviewUrl(item)
                    const itemHref = getResultHref(getImageSiteHref(item) || previewSrc)
                    const imageHref = getResultHref(previewSrc)
                    const siteHref = getResultHref(getImageSiteHref(item))
                    const siteName = item?.site?.title || item?.site?.name || item?.site?.url || getDisplayUrl(siteHref)
                    const actionKey = `${imageHref}-${siteHref}-${index}`
                    const domainLabel = getImageDomainLabel(siteHref)

                    return (
                      <article className="image-result-item" key={`${previewSrc}-${index}`}>
                        <button
                          className="image-result-link"
                          type="button"
                          onClick={() => openPreview(item)}
                          aria-label={`Preview ${item.title || 'image result'}`}
                        >
                          <img className="image-result-image" src={previewSrc} alt={item.title || 'Search result image'} />
                        </button>

                        <div className="image-result-meta">
                          <div className="image-result-meta-row">
                            <a className="image-result-site" href={siteHref} target="_blank" rel="noreferrer">
                              {siteName && <span className="image-result-title">{siteName}</span>}
                              <span className="image-result-site-badge">
                                <ResultFavicon
                                  src={getFaviconSource(item, siteHref)}
                                  alt={`${siteName} favicon`}
                                  fallbackText={getFallbackText(siteHref || previewSrc)}
                                />
                                <span className="image-result-domain">{domainLabel}</span>
                              </span>
                            </a>

                            <div className="result-action-wrapper">
                              <button
                                className="result-action"
                                type="button"
                                onClick={(event) => {
                                  event.stopPropagation()
                                  toggleResultMenu(actionKey)
                                }}
                                aria-label="Open image actions"
                              >
                                {copiedUrl === imageHref || copiedUrl === siteHref ? '✓' : '⋯'}
                              </button>

                              {openMenuUrl === actionKey && (
                                <div className="result-action-menu" role="menu">
                                  <button
                                    className="result-action-menu-item"
                                    type="button"
                                    onClick={(event) => {
                                      event.stopPropagation()
                                      copyLink(imageHref)
                                    }}
                                    role="menuitem"
                                  >
                                    Copy image link
                                  </button>
                                  <button
                                    className="result-action-menu-item"
                                    type="button"
                                    onClick={(event) => {
                                      event.stopPropagation()
                                      copyLink(siteHref)
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
                    )
                  })}
                </div>
              ) : (
                visibleResults.map((item) => {
                  const itemHref = getResultHref(item.url)

                  return (
                    <article className="result-item" key={`${item.title}-${item.url}`}>
                      <div className="result-favicon-wrap">
                        <ResultFavicon
                          src={getFaviconSource(item, itemHref)}
                          alt={`${item.title} favicon`}
                          fallbackText={getFallbackText(item.url)}
                        />
                      </div>

                      <div className="result-content">
                        <a className="result-url" href={itemHref} target="_blank" rel="noreferrer">
                          {getDisplayUrl(item.url)}
                        </a>
                        <a className="result-title-link" href={itemHref} target="_blank" rel="noreferrer">
                          {item.title}
                        </a>
                        <p className="result-description">{item.description}</p>
                      </div>

                      <div className="result-action-wrapper">
                        <button
                          className="result-action"
                          type="button"
                          onClick={(event) => {
                            event.stopPropagation()
                            toggleResultMenu(itemHref)
                          }}
                          aria-label="Open result actions"
                        >
                          {copiedUrl === itemHref ? '✓' : '⋯'}
                        </button>

                        {openMenuUrl === itemHref && (
                          <div className="result-action-menu" role="menu">
                            <button
                              className="result-action-menu-item"
                              type="button"
                              onClick={(event) => {
                                event.stopPropagation()
                                copyLink(itemHref)
                              }}
                              role="menuitem"
                            >
                              Copy link
                            </button>
                          </div>
                        )}
                      </div>
                    </article>
                  )
                })
              )}
            </div>
          </section>
        )}
      </main>

      {previewImage && (
        <div className="image-preview-overlay" role="dialog" aria-modal="true" onClick={closePreview}>
          <div className="image-preview-card" onClick={(event) => event.stopPropagation()}>
            <button className="image-preview-close" type="button" onClick={closePreview} aria-label="Close preview">
              ×
            </button>
            <img className="image-preview-image" src={previewImage.src} alt={previewImage.alt} />
          </div>
        </div>
      )}
    </div>
  )
}

export default App
