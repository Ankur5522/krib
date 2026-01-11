import { useState } from "react";
import { MessageSquare, Phone } from "lucide-react";
import { apiGet } from "../lib/api";

interface ContactRevealProps {
  postId: string;
  theme: any;
}

export const ContactReveal = ({ postId, theme }: ContactRevealProps) => {
  const [phone, setPhone] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isMobile = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent);

  const handleContactClick = async () => {
    if (phone) {
      // Phone already loaded
      if (isMobile) {
        window.location.href = `tel:${phone}`;
      }
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const data = await apiGet<{ phone: string }>(`/api/contact/${postId}`);
      const phoneNumber = data.phone;
      setPhone(phoneNumber);

      // If mobile, trigger tel: immediately
      if (isMobile) {
        setTimeout(() => {
          window.location.href = `tel:${phoneNumber}`;
        }, 100);
      }
    } catch (err) {
      let errorMessage = "Failed to load contact";

      if (err instanceof Error) {
        // Try to parse error response
        try {
          const jsonMatch = err.message.match(/\{.*\}/);
          if (jsonMatch) {
            const errorData = JSON.parse(jsonMatch[0]);

            // Handle specific error cases
            if (errorData.error === "No contact information available") {
              errorMessage = "No contact info provided";
            } else if (errorData.error === "Message not found") {
              errorMessage = "Message not found";
            } else if (errorData.message) {
              errorMessage = errorData.message;
            } else if (errorData.error) {
              errorMessage = errorData.error;
            }
          } else {
            // Clean up error message
            const cleanError = err.message.replace(/HTTP \d+:/, "").trim();
            if (cleanError.includes("No contact information")) {
              errorMessage = "No contact info provided";
            } else {
              errorMessage = cleanError;
            }
          }
        } catch {
          // If parsing fails, use the original error message
          const cleanError = err.message.replace(/HTTP \d+:/, "").trim();
          errorMessage = cleanError.includes("No contact information")
            ? "No contact info provided"
            : cleanError;
        }
      }

      setError(errorMessage);
      console.error(err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleWhatsAppClick = () => {
    if (!phone) return;
    const text = "Hi, I saw your post on Kirb...";
    const url = `https://wa.me/${phone}?text=${encodeURIComponent(text)}`;
    window.open(url, "_blank");
  };

  const handleCallClick = () => {
    if (!phone) return;
    window.location.href = `tel:${phone}`;
  };

  if (error) {
    return (
      <button
        disabled
        className="w-full px-3 py-2 text-red-600 bg-red-100/80 rounded-lg text-xs font-semibold"
      >
        {error}
      </button>
    );
  }

  if (phone) {
    return (
      <div className="w-full space-y-2">
        <div
          className={`text-xs font-semibold px-3 py-2 ${theme.accentSoft} rounded-lg text-center`}
        >
          {phone}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleWhatsAppClick}
            className="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-full font-semibold text-xs bg-green-500 hover:bg-green-600 text-white transition-all active:scale-95"
          >
            <MessageSquare className="w-3.5 h-3.5" />
            <span>WhatsApp</span>
          </button>
          {isMobile && (
            <button
              onClick={handleCallClick}
              className="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-full font-semibold text-xs bg-blue-500 hover:bg-blue-600 text-white transition-all active:scale-95"
            >
              <Phone className="w-3.5 h-3.5" />
              <span>Call</span>
            </button>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-center justify-end gap-2">
      <button
        onClick={handleContactClick}
        disabled={isLoading}
        className={`flex items-center gap-1.5 px-4 py-2 rounded-full bg-white text-black text-xs font-semibold transition-all hover:opacity-90 ${
          isLoading ? "opacity-50 cursor-not-allowed" : "cursor-pointer"
        }`}
      >
        <MessageSquare className="w-3.5 h-3.5" />
        <span>{isLoading ? "Loading..." : "Contact"}</span>
      </button>
    </div>
  );
};
