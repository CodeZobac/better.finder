import React from 'react';
import { SearchResult, ResultType } from '../types';
import { File, AppWindow, Zap, Calculator, Clipboard, Bookmark, Clock, Globe } from 'lucide-react';

interface ResultItemProps {
  result: SearchResult;
  isSelected: boolean;
  onSelect: () => void;
  onExecute: () => void;
  highlightedTitle?: React.ReactNode;
}

const ResultItem: React.FC<ResultItemProps> = ({
  result,
  isSelected,
  onSelect,
  onExecute,
  highlightedTitle,
}) => {
  const itemRef = React.useRef<HTMLDivElement>(null);

  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    // Create ripple effect
    if (itemRef.current) {
      const rect = itemRef.current.getBoundingClientRect();
      const ripple = document.createElement('span');
      const size = Math.max(rect.width, rect.height);
      const x = e.clientX - rect.left - size / 2;
      const y = e.clientY - rect.top - size / 2;

      ripple.style.width = ripple.style.height = `${size}px`;
      ripple.style.left = `${x}px`;
      ripple.style.top = `${y}px`;
      ripple.classList.add('ripple');

      itemRef.current.appendChild(ripple);

      setTimeout(() => {
        ripple.remove();
      }, 600);
    }

    onSelect();
    onExecute();
  };

  const handleMouseEnter = () => {
    onSelect();
  };

  // Get icon based on result type
  const getIcon = () => {
    const iconClass = "w-8 h-8 flex-shrink-0";
    
    if (result.icon) {
      // If icon is a base64 encoded image
      if (result.icon.startsWith('data:image')) {
        return <img src={result.icon} alt="" className={iconClass} />;
      }
      // If icon is a URL
      return <img src={result.icon} alt="" className={iconClass} />;
    }

    // Default icons based on type
    switch (result.type) {
      case ResultType.File:
        return <File className={iconClass + " text-primary"} />;
      case ResultType.Application:
        return <AppWindow className={iconClass + " text-primary"} />;
      case ResultType.QuickAction:
        return <Zap className={iconClass + " text-primary"} />;
      case ResultType.Calculator:
        return <Calculator className={iconClass + " text-primary"} />;
      case ResultType.Clipboard:
        return <Clipboard className={iconClass + " text-primary"} />;
      case ResultType.Bookmark:
        return <Bookmark className={iconClass + " text-primary"} />;
      case ResultType.RecentFile:
        return <Clock className={iconClass + " text-primary"} />;
      case ResultType.WebSearch:
        return <Globe className={iconClass + " text-primary"} />;
      default:
        return <File className={iconClass + " text-primary"} />;
    }
  };

  // Get badge text based on result type
  const getBadgeText = () => {
    switch (result.type) {
      case ResultType.File:
        return 'File';
      case ResultType.Application:
        return 'App';
      case ResultType.QuickAction:
        return 'Action';
      case ResultType.Calculator:
        return 'Calc';
      case ResultType.Clipboard:
        return 'Clip';
      case ResultType.Bookmark:
        return 'Bookmark';
      case ResultType.RecentFile:
        return 'Recent';
      case ResultType.WebSearch:
        return 'Web';
      default:
        return '';
    }
  };

  return (
    <div
      ref={itemRef}
      className={`
        flex items-center gap-3 px-4 py-3 cursor-pointer
        ripple-container interactive
        ${isSelected 
          ? 'bg-surface-hover translate-x-1' 
          : 'hover:bg-surface-hover hover:translate-x-1'
        }
      `}
      onClick={handleClick}
      onMouseEnter={handleMouseEnter}
      role="option"
      aria-selected={isSelected}
    >
      {/* Icon */}
      <div className="flex-shrink-0">
        {getIcon()}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-medium text-text-primary truncate">
            {highlightedTitle || result.title}
          </h3>
          <span className="text-xs px-1.5 py-0.5 rounded bg-primary/20 text-primary flex-shrink-0">
            {getBadgeText()}
          </span>
        </div>
        {result.subtitle && (
          <p className="text-xs text-text-secondary truncate mt-0.5">
            {result.subtitle}
          </p>
        )}
      </div>
    </div>
  );
};

export default ResultItem;
