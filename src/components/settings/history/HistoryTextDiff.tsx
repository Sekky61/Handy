import React from "react";

type DiffSegment = {
  text: string;
  status: "unchanged" | "added" | "removed";
};

export type TextDiff = {
  original: DiffSegment[];
  processed: DiffSegment[];
};

const tokenizeForDiff = (text: string): string[] => {
  return text.match(/\s+|[\p{L}\p{N}_]+|[^\s\p{L}\p{N}_]/gu) ?? [];
};

export const buildTextDiff = (
  original: string,
  processed: string,
): TextDiff => {
  const originalTokens = tokenizeForDiff(original);
  const processedTokens = tokenizeForDiff(processed);
  const rows = originalTokens.length + 1;
  const columns = processedTokens.length + 1;
  const lengths = Array.from({ length: rows }, () => Array(columns).fill(0));

  for (let i = originalTokens.length - 1; i >= 0; i -= 1) {
    for (let j = processedTokens.length - 1; j >= 0; j -= 1) {
      lengths[i][j] =
        originalTokens[i] === processedTokens[j]
          ? lengths[i + 1][j + 1] + 1
          : Math.max(lengths[i + 1][j], lengths[i][j + 1]);
    }
  }

  const originalDiff: DiffSegment[] = [];
  const processedDiff: DiffSegment[] = [];
  let i = 0;
  let j = 0;

  const pushSegment = (target: DiffSegment[], segment: DiffSegment) => {
    const previous = target[target.length - 1];
    if (previous?.status === segment.status) {
      previous.text += segment.text;
      return;
    }
    target.push(segment);
  };

  while (i < originalTokens.length && j < processedTokens.length) {
    if (originalTokens[i] === processedTokens[j]) {
      pushSegment(originalDiff, {
        text: originalTokens[i],
        status: "unchanged",
      });
      pushSegment(processedDiff, {
        text: processedTokens[j],
        status: "unchanged",
      });
      i += 1;
      j += 1;
    } else if (lengths[i + 1][j] >= lengths[i][j + 1]) {
      pushSegment(originalDiff, { text: originalTokens[i], status: "removed" });
      i += 1;
    } else {
      pushSegment(processedDiff, { text: processedTokens[j], status: "added" });
      j += 1;
    }
  }

  while (i < originalTokens.length) {
    pushSegment(originalDiff, { text: originalTokens[i], status: "removed" });
    i += 1;
  }

  while (j < processedTokens.length) {
    pushSegment(processedDiff, { text: processedTokens[j], status: "added" });
    j += 1;
  }

  return { original: originalDiff, processed: processedDiff };
};

export const DiffText: React.FC<{
  segments: DiffSegment[];
  variant: "original" | "processed";
}> = ({ segments, variant }) => (
  <>
    {segments.map((segment, index) => {
      const className =
        segment.status === "removed" && variant === "original"
          ? "rounded-sm bg-red-500/10 text-red-500 line-through decoration-red-500/80 decoration-2"
          : segment.status === "added" && variant === "processed"
            ? "rounded-sm bg-logo-primary/10 text-logo-primary underline decoration-logo-primary/80 decoration-2 underline-offset-2"
            : undefined;

      return (
        <span key={`${index}-${segment.status}`} className={className}>
          {segment.text}
        </span>
      );
    })}
  </>
);
