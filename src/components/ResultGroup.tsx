import React, { useMemo, useRef, useEffect } from 'react';
import { List, ListImperativeAPI } from 'react-window';
import { SearchResult, ResultType } from '../types';
import ResultItem from './ResultItem';
import { highlightText } from '../utils/highlight';

interface ResultGroupProps {
  results: SearchResult[];
  selectedIndex: number;
  onSelectResult: (index: number) => void;
  onExecuteResult: (index: number) => void;
  query?: string;
}

interface GroupedResults {
  type: ResultType;
  title: string;
  results: SearchResult[];
  startIndex: number;
}

interface FlattenedItem {
  type: 'header' | 'result' | 'separator';
  data?: SearchResult;
  title?: string;
  absoluteIndex?: number;
}

// Constants for virtual scrolling
const ITEM_HEIGHT = 56; // Height of result item in pixels
const HEADER_HEIGHT = 32; // Height of group header in pixels
const SEPARATOR_HEIGHT = 9; // Height of separator (margin + border)
const MAX_HEIGHT = 480; // Maximum height of results container
const OVERSCAN_COUNT = 3; // Number of items to render outside visible area

const ResultGroup: React.FC<ResultGroupProps> = ({
  results,
  selectedIndex,
  onSelectResult,
  onExecuteResult,
  query,
}) => {
  // Group results by type
  const groupedResults = useMemo(() => {
    const groups: GroupedResults[] = [];
    const typeMap = new Map<ResultType, SearchResult[]>();

    // Group results by type
    results.forEach((result) => {
      const existing = typeMap.get(result.type);
      if (existing) {
        existing.push(result);
      } else {
        typeMap.set(result.type, [result]);
      }
    });

    // Convert to array with start indices
    let currentIndex = 0;
    const typeOrder = [
      ResultType.RecentFile,
      ResultType.File,
      ResultType.Application,
      ResultType.QuickAction,
      ResultType.Calculator,
      ResultType.Bookmark,
      ResultType.Clipboard,
      ResultType.WebSearch,
    ];

    typeOrder.forEach((type) => {
      const typeResults = typeMap.get(type);
      if (typeResults && typeResults.length > 0) {
        groups.push({
          type,
          title: getGroupTitle(type),
          results: typeResults,
          startIndex: currentIndex,
        });
        currentIndex += typeResults.length;
      }
    });

    return groups;
  }, [results]);

  // Get group title based on type
  function getGroupTitle(type: ResultType): string {
    switch (type) {
      case ResultType.File:
        return 'FILES';
      case ResultType.Application:
        return 'APPLICATIONS';
      case ResultType.QuickAction:
        return 'QUICK ACTIONS';
      case ResultType.Calculator:
        return 'CALCULATOR';
      case ResultType.Clipboard:
        return 'CLIPBOARD';
      case ResultType.Bookmark:
        return 'BOOKMARKS';
      case ResultType.RecentFile:
        return 'RECENT FILES';
      case ResultType.WebSearch:
        return 'WEB SEARCH';
      default:
        return 'RESULTS';
    }
  }

  // Flatten results into a single list with headers and separators
  const flattenedItems = useMemo(() => {
    const items: FlattenedItem[] = [];
    
    groupedResults.forEach((group, groupIndex) => {
      // Add group header
      items.push({
        type: 'header',
        title: group.title,
      });

      // Add results
      group.results.forEach((result, resultIndex) => {
        const absoluteIndex = group.startIndex + resultIndex;
        items.push({
          type: 'result',
          data: result,
          absoluteIndex,
        });
      });

      // Add separator (except for last group)
      if (groupIndex < groupedResults.length - 1) {
        items.push({
          type: 'separator',
        });
      }
    });

    return items;
  }, [groupedResults]);

  // Calculate item size based on type
  const getItemSize = (index: number): number => {
    const item = flattenedItems[index];
    if (item.type === 'header') return HEADER_HEIGHT;
    if (item.type === 'separator') return SEPARATOR_HEIGHT;
    return ITEM_HEIGHT;
  };

  // Calculate total height
  const totalHeight = useMemo(() => {
    return flattenedItems.reduce((sum, _item, index) => sum + getItemSize(index), 0);
  }, [flattenedItems]);

  const listHeight = Math.min(totalHeight, MAX_HEIGHT);

  // Ref for the list to scroll to selected item
  const listRef = useRef<ListImperativeAPI>(null);

  // Scroll to selected item when selection changes
  useEffect(() => {
    if (listRef.current && selectedIndex >= 0) {
      // Find the flattened index of the selected result
      const flatIndex = flattenedItems.findIndex(
        (item) => item.type === 'result' && item.absoluteIndex === selectedIndex
      );
      if (flatIndex >= 0) {
        listRef.current.scrollToRow({ index: flatIndex, align: 'smart' });
      }
    }
  }, [selectedIndex, flattenedItems]);

  if (results.length === 0) {
    return null;
  }

  // Render individual row in virtual list - using useMemo to create a stable component
  const Row = useMemo(() => {
    return ({ index, style }: { index: number; style: React.CSSProperties }) => {
      const item = flattenedItems[index];

      if (item.type === 'header') {
        return (
          <div style={style} className="px-4 py-2 text-xs font-semibold text-text-secondary uppercase tracking-wide">
            {item.title}
          </div>
        );
      }

      if (item.type === 'separator') {
        return (
          <div style={style} className="flex items-center">
            <div className="mx-4 flex-1 border-t border-border" />
          </div>
        );
      }

      // Result item
      const result = item.data!;
      const absoluteIndex = item.absoluteIndex!;
      const highlightedTitle = query ? highlightText(result.title, query) : undefined;

      return (
        <div style={style}>
          <ResultItem
            result={result}
            isSelected={absoluteIndex === selectedIndex}
            onSelect={() => onSelectResult(absoluteIndex)}
            onExecute={() => onExecuteResult(absoluteIndex)}
            highlightedTitle={highlightedTitle}
          />
        </div>
      );
    };
  }, [flattenedItems, query, selectedIndex, onSelectResult, onExecuteResult]);

  // Render results with virtual scrolling
  return (
    <div className="py-2">
      <List
        listRef={listRef}
        defaultHeight={listHeight}
        rowCount={flattenedItems.length}
        rowHeight={getItemSize}
        rowComponent={Row as any}
        rowProps={{} as any}
        overscanCount={OVERSCAN_COUNT}
      />
    </div>
  );
};

export default ResultGroup;
