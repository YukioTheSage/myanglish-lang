"use client";
import { motion } from "framer-motion";

interface CardProps {
  children: React.ReactNode;
  className?: string;
  delay?: number;
  hoverGlow?: boolean;
}

export default function Card({ children, className = "", delay = 0, hoverGlow = true }: CardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, delay: delay * 0.1 }}
      whileHover={hoverGlow ? {
        backgroundColor: "rgba(31, 43, 61, 1)",
        borderColor: "rgba(96, 165, 250, 0.3)",
        boxShadow: "0 0 30px rgba(96, 165, 250, 0.15)",
        y: -2,
      } : undefined}
      className={`bg-bg-card border border-white/10 rounded-xl p-5 transition-all ${className}`}
    >
      {children}
    </motion.div>
  );
}
