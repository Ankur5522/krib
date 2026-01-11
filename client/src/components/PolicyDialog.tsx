import React, { useState } from "react";
import { CheckCircle } from "lucide-react";

interface PolicyDialogProps {
  onAccept: () => void;
  theme?: Record<string, string>;
  darkMode?: boolean;
}

const PolicyDialog: React.FC<PolicyDialogProps> = ({ onAccept, theme }) => {
  const [open, setOpen] = useState(true);

  // Default theme if not provided
  const defaultTheme = {
    bgCard: "bg-white",
    bgTertiary: "bg-zinc-100",
    border: "border-zinc-200",
    text: "text-black",
    textSecondary: "text-zinc-600",
    textMuted: "text-zinc-400",
    accent: "bg-black text-white",
    accentSoft: "bg-zinc-100",
    input: "bg-zinc-100 border-zinc-200 text-black placeholder-zinc-400",
    glow: "shadow-lg",
  };

  const activeTheme = theme || defaultTheme;

  const handleAccept = () => {
    setOpen(false);
    onAccept();
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div
        className={`${activeTheme.bgCard} ${activeTheme.border} border rounded-2xl p-8 max-w-lg w-full ${activeTheme.glow} relative`}
      >
        <CheckCircle className="w-16 h-16 mx-auto mb-4 text-blue-500" />
        <h2
          className={`text-2xl font-bold mb-6 ${activeTheme.text} text-center`}
        >
          Policies & Consent
        </h2>
        <div
          className={`overflow-y-auto max-h-96 text-sm ${activeTheme.textSecondary} mb-6 space-y-4`}
        >
          <section>
            <h3 className="font-semibold">Terms of Use (IT Rules 2021)</h3>
            <ul className="list-disc ml-5">
              <li>
                RoomShout is an <b>Intermediary</b> under Section 79 of the IT
                Act. Users are prohibited from posting content that is
                defamatory, obscene, harmful to minors, impersonates others, or
                threatens the sovereignty of India.
              </li>
              <li>
                We reserve the right to remove any post that violates these
                rules or upon government notice.
              </li>
              <li>
                <b>Not an Agent:</b> We do not verify listings or participate in
                negotiations. We are not a RERA-regulated agent as we do not
                charge brokerage or facilitate sales.
              </li>
            </ul>
          </section>

          <section>
            <h3 className="font-semibold">Privacy (DPDP Act 2023)</h3>
            <ul className="list-disc ml-5">
              <li>
                <b>Consent:</b> By clicking "I Accept", you provide affirmative
                consent for us to process your IP and Device Fingerprint
                strictly for fraud prevention.
              </li>
              <li>
                <b>Data Minimization:</b> Public posts are visible for 48 hours.
                We do not sell your data to third parties.
              </li>
              <li>
                <b>Right to Erase:</b> You may request the removal of your post
                at any time via the Grievance Officer.
              </li>
            </ul>
          </section>
        </div>
        <button
          className={`${activeTheme.accent} w-full px-6 py-3 rounded-full font-semibold transition-all hover:opacity-90`}
          onClick={handleAccept}
        >
          I Accept
        </button>
      </div>
    </div>
  );
};

export default PolicyDialog;
