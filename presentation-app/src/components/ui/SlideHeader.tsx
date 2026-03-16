"use client";
import { motion } from "framer-motion";

interface SlideHeaderProps {
  number: string;
  title: string;
  badge?: string;
  badgeType?: "default" | "file";
  centered?: boolean;
}

export default function SlideHeader({ number, title, badge, badgeType = "default", centered = false }: SlideHeaderProps) {
  const badgeClasses = badgeType === "file"
    ? "bg-accent-blue/10 text-accent-blue border-accent-blue/20 font-mono normal-case tracking-normal"
    : "bg-accent-purple/15 text-accent-purple border-accent-purple/25";

  return (
    <motion.div
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
      className={`flex flex-wrap items-center gap-2 sm:gap-4 mb-6 sm:mb-8 ${centered ? "justify-center" : ""}`}
    >
      <span className="text-[0.72rem] sm:text-[0.8rem] font-bold font-mono text-accent-blue bg-accent-blue/10 border border-accent-blue/20 px-2.5 py-1 rounded-md tracking-wide">
        {number}
      </span>
      <h2 className="text-[1.35rem] sm:text-[2rem] font-extrabold tracking-tight gradient-text">
        {title}
      </h2>
      {badge && (
        <span className={`text-[0.66rem] sm:text-[0.7rem] font-semibold uppercase tracking-wider px-3 py-1.5 rounded-full border ${badgeClasses}`}>
          {badge}
        </span>
      )}
    </motion.div>
  );
}
