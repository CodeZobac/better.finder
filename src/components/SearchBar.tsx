import React, { useState, useEffect, useRef } from 'react';
import { Search, X, Settings } from 'lucide-react';
import { useKeyboard, useKeyboardSelection } from '../hooks/useKeyboard';
import { useSearch } from '../hooks/useSearch';
import ResultGroup from './ResultGroup';
import ResultSkeleton from './ResultSkeleton';

interface SearchBarProps {
  isVisible: boolean;
  onClose: () => void;
  onOpenSettings?: () => void;
}

const SearchBar: React.FC<SearchBarProps> = ({ isVisible, onClose, onOpenSettings }) => {
  const [isAnimatingOut, setIsAnimatingOut] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  
  // Use the search hook
  const { query, setQuery, results, isLoading, executeResult } = useSearch();

  // Keyboard selection management
  const {
    selectedIndex,
    moveUp,
    moveDown,
    resetSelection,
  } = useKeyboardSelection(results.length);

  // Handle close with animation
  const handleClose = () => {
    setIsAnimatingOut(true);
    setTimeout(() => {
      onClose();
      setIsAnimatingOut(false);
    }, 100); // Match animation duration
  };

  // Handle result execution
  const handleExecuteResult = async () => {
    if (results.length > 0 && selectedIndex >= 0 && selectedIndex < results.length) {
      const selectedResult = results[selectedIndex];
      try {
        await executeResult(selectedResult);
        handleClose();
      } catch (error) {
        console.error('Failed to execute result:', error);
      }
    }
  };

  // Auto-focus on mount and when visibility changes
  useEffect(() => {
    if (isVisible) {
      setIsAnimatingOut(false);
      setQuery(''); // Clear query when window opens
      resetSelection();
      if (inputRef.current) {
        inputRef.current.focus();
      }
    }
  }, [isVisible, resetSelection]);

  // Keyboard navigation
  useKeyboard({
    onArrowUp: moveUp,
    onArrowDown: moveDown,
    onEnter: handleExecuteResult,
    onEscape: handleClose,
    enabled: isVisible,
  });

  // Handle input change
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setQuery(value);
    resetSelection(); // Reset selection when query changes
  };

  // Handle clear button
  const handleClear = () => {
    setQuery('');
    resetSelection();
    if (inputRef.current) {
      inputRef.current.focus();
    }
  };

  if (!isVisible) {
    return null;
  }

  return (
    <div 
      ref={containerRef}
      className={`w-full max-w-[600px] bg-background border border-border rounded-[12px] overflow-hidden ${
        isAnimatingOut ? 'animate-fade-out' : 'animate-slide-in'
      }`}
      style={{ boxShadow: 'var(--shadow-window)' }}
    >
      {/* Search Input */}
      <div className="flex items-center gap-3 px-4 py-3 border-b border-border bg-surface">
        <Search className="w-5 h-5 flex-shrink-0" style={{ color: 'var(--color-text-secondary)' }} />
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={handleInputChange}
          placeholder="Search files, apps, and more..."
          className="flex-1 bg-transparent text-base outline-none placeholder:text-text-secondary"
          style={{ color: 'var(--color-text-primary)' }}
          autoComplete="off"
          spellCheck={false}
        />
        {query && (
          <button
            onClick={handleClear}
            className="p-1 hover:bg-surface-hover rounded transition-colors"
            aria-label="Clear search"
          >
            <X className="w-4 h-4 text-text-secondary" />
          </button>
        )}
        {onOpenSettings && (
          <button
            onClick={() => {
              onOpenSettings();
              handleClose();
            }}
            className="p-1 hover:bg-surface-hover rounded transition-colors"
            aria-label="Open settings"
            title="Settings"
          >
            <Settings className="w-4 h-4 text-text-secondary" />
          </button>
        )}
      </div>

      {/* Results */}
      <div className="max-h-[400px] overflow-y-auto smooth-scroll bg-background">
        {isLoading && (
          <ResultSkeleton count={5} />
        )}
        
        {!isLoading && query && results.length === 0 && (
          <div className="px-4 py-8 text-center animate-fade-in" style={{ color: 'var(--color-text-secondary)' }}>
            No results found
          </div>
        )}

        {!isLoading && results.length > 0 && (
          <div className="animate-fade-in">
            <ResultGroup
              results={results}
              selectedIndex={selectedIndex}
              onSelectResult={() => {
                // Selection is handled by keyboard hook and mouse hover
              }}
              onExecuteResult={handleExecuteResult}
              query={query}
            />
          </div>
        )}
        
        {!isLoading && !query && (
          <div className="px-4 py-8 text-center animate-fade-in" style={{ color: 'var(--color-text-secondary)' }}>
            Type to search files, apps, and more...
          </div>
        )}
      </div>
    </div>
  );
};

export default SearchBar;
