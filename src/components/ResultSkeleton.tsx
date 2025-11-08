import React from 'react';

interface ResultSkeletonProps {
  count?: number;
}

const ResultSkeleton: React.FC<ResultSkeletonProps> = ({ count = 5 }) => {
  return (
    <div className="py-2">
      {Array.from({ length: count }).map((_, index) => (
        <div
          key={index}
          className="flex items-center gap-3 px-4 py-3 animate-fade-in"
          style={{ animationDelay: `${index * 50}ms` }}
        >
          {/* Icon skeleton */}
          <div className="w-8 h-8 rounded skeleton flex-shrink-0" />
          
          {/* Content skeleton */}
          <div className="flex-1 space-y-2">
            <div className="h-4 bg-surface-hover rounded skeleton w-3/4" />
            <div className="h-3 bg-surface-hover rounded skeleton w-1/2" />
          </div>
        </div>
      ))}
    </div>
  );
};

export default ResultSkeleton;
