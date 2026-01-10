import { useState } from "react";
import { MessageSquare, Phone } from "lucide-react";
import { cn } from "../lib/utils";
import { apiGet } from "../lib/api";

interface ContactRevealProps {
  postId: string;
}

export const ContactReveal = ({ postId }: ContactRevealProps) => {
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
        // Try to parse rate limit error
        try {
          const jsonMatch = err.message.match(/\{.*\}/);
          if (jsonMatch) {
            const errorData = JSON.parse(jsonMatch[0]);
            errorMessage = errorData.message || err.message;
          } else {
            errorMessage = err.message;
          }
        } catch {
          errorMessage = err.message;
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
        className="w-full px-3 py-2 text-red-600 bg-red-100 rounded-lg text-sm font-semibold"
      >
        {error}
      </button>
    );
  }

  if (phone) {
    return (
      <div className="w-full space-y-2">
        <div className="text-xs font-semibold text-gray-600 px-3 py-2 bg-gray-100 rounded-lg text-center">
          {phone}
        </div>
        <button
          onClick={handleWhatsAppClick}
          className="w-full flex items-center justify-center gap-2 py-2 px-3 rounded-lg font-semibold text-sm bg-green-500 hover:bg-green-600 text-white transition-all active:scale-95"
        >
          <MessageSquare size={16} />
          <span>WhatsApp</span>
        </button>
        {isMobile && (
          <button
            onClick={handleCallClick}
            className="w-full flex items-center justify-center gap-2 py-2 px-3 rounded-lg font-semibold text-sm bg-blue-500 hover:bg-blue-600 text-white transition-all active:scale-95"
          >
            <Phone size={16} />
            <span>Call</span>
          </button>
        )}
      </div>
    );
  }

  return (
    <button
      onClick={handleContactClick}
      disabled={isLoading}
      className={cn(
        "w-full flex items-center justify-center gap-2 py-2 px-3 rounded-lg font-semibold text-sm text-white transition-all active:scale-95",
        isLoading
          ? "bg-gray-400 cursor-not-allowed"
          : "bg-blue-500 hover:bg-blue-600"
      )}
    >
      <MessageSquare size={16} />
      <span>{isLoading ? "Loading..." : "Contact"}</span>
    </button>
  );
};
