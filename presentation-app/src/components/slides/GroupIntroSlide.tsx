"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import Card from "../ui/Card";

const members = [
  { slot: "Member 1", name: "Phone Sett Paing Kyaw", detail: "Student ID: 6708369" },
  { slot: "Member 2", name: "Bhone Pyae Hein", detail: "Student ID: 6708381" },
  { slot: "Member 3", name: "Thuta Soe", detail: "Student ID: 6708211" },
  { slot: "Presenter", name: "Nyan Lin Htet", detail: "Student ID: 6708397", highlight: true },
  { slot: "Member 5", name: "Han Phyo Htet", detail: "Student ID: 6708463" },
];

export default function GroupIntroSlide() {
  return (
    <div>
      <SlideHeader number="02" title="Group Introduction" />
      <div className="grid grid-cols-1 gap-6">
        <Card className="h-full">
          <p className="text-[0.72rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Project Snapshot</p>
          <div className="space-y-3 text-[0.9rem]">
            <div>
              <span className="block text-text-muted text-[0.72rem] uppercase tracking-wide mb-1">Project</span>
              <span className="font-semibold">M-Lang: Myanglish Programming Language and Compiler</span>
            </div>
            <div>
              <span className="block text-text-muted text-[0.72rem] uppercase tracking-wide mb-1">Focus</span>
              <span className="text-text-secondary">Programming language design, compiler construction, and developer tooling</span>
            </div>
            <div>
              <span className="block text-text-muted text-[0.72rem] uppercase tracking-wide mb-1">Institution</span>
              <span className="text-text-secondary">Rangsit University</span>
            </div>
            <div>
              <span className="block text-text-muted text-[0.72rem] uppercase tracking-wide mb-1">Presentation Goal</span>
              <span className="text-text-secondary">Review the reference language, explain the new language design, and show the implemented compiler pipeline</span>
            </div>
          </div>
        </Card>

        <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-5 gap-4">
          {members.map((member, index) => (
            <motion.div
              key={member.slot}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.1 }}
              className={`rounded-xl p-5 border ${
                member.highlight
                  ? "bg-accent-blue/[0.08] border-accent-blue/25 shadow-[0_0_24px_rgba(96,165,250,0.12)]"
                  : "bg-bg-card border-white/10"
              }`}
            >
              <span className={`block text-[0.7rem] font-bold uppercase tracking-[0.15em] mb-3 ${member.highlight ? "text-accent-blue" : "text-accent-purple"}`}>
                {member.slot}
              </span>
              <h3 className="text-lg font-bold mb-2">{member.name}</h3>
              <p className="text-[0.82rem] text-text-secondary leading-relaxed">{member.detail}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </div>
  );
}
