import React from 'react';
import Fuzzysort from 'fuzzysort';

/**
 * Highlights matched characters in a search result using fuzzysort
 * @param result - Fuzzysort result object
 * @returns React node with highlighted characters
 */
export function highlightMatches(result: Fuzzysort.Result | null): React.ReactNode {
  if (!result || !result.target) {
    return null;
  }

  // If no highlights, return plain text
  if (!result.indexes || result.indexes.length === 0) {
    return result.target;
  }

  const text = result.target;
  const indexes = result.indexes;
  const elements: React.ReactNode[] = [];
  let lastIndex = 0;

  indexes.forEach((index, i) => {
    // Add non-highlighted text before this match
    if (index > lastIndex) {
      elements.push(
        <span key={`text-${i}`}>
          {text.substring(lastIndex, index)}
        </span>
      );
    }

    // Add highlighted character
    elements.push(
      <span key={`highlight-${i}`} className="text-primary font-semibold">
        {text[index]}
      </span>
    );

    lastIndex = index + 1;
  });

  // Add remaining text after last match
  if (lastIndex < text.length) {
    elements.push(
      <span key="text-end">
        {text.substring(lastIndex)}
      </span>
    );
  }

  return <>{elements}</>;
}

/**
 * Highlights matched characters in a plain string
 * @param text - The text to highlight
 * @param query - The search query
 * @returns React node with highlighted characters
 */
export function highlightText(text: string, query: string): React.ReactNode {
  if (!query || !text) {
    return text;
  }

  const result = Fuzzysort.single(query, text);
  return highlightMatches(result);
}
