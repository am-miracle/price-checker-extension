import React from "react";

interface SearchIconProps {
  size?: number;
  color?: string;
  className?: string;
}

const SearchIcon: React.FC<SearchIconProps> = ({
  size = 20,
  color = "#B9B9B9",
  className,
}) => {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 20 20"
      fill={color}
      className={className}
    >
      <path d="M12.9 14.32a8 8 0 1 1 1.41-1.41l5.35 5.33l-1.42 1.42l-5.33-5.34zM8 14A6 6 0 1 0 8 2a6 6 0 0 0 0 12z" />
    </svg>
  );
};

export default SearchIcon;
