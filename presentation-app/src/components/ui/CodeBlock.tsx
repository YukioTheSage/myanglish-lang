"use client";
import { motion } from "framer-motion";

interface CodeBlockProps {
  children: React.ReactNode;
  className?: string;
  size?: "small" | "normal" | "large";
}

export default function CodeBlock({ children, className = "", size = "normal" }: CodeBlockProps) {
  const sizeClasses = {
    small: "text-[0.75rem] p-4 leading-[1.6]",
    normal: "text-[0.8rem] p-5 leading-[1.7]",
    large: "text-[0.9rem] p-8 leading-[1.7]",
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, delay: 0.2 }}
      data-slide-nav-lock-x
      className={`code-block bg-bg-code border border-white/10 rounded-xl overflow-x-auto touch-pan-x ${sizeClasses[size]} ${className}`}
    >
      {children}
    </motion.div>
  );
}
