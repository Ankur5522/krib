import React from "react";

const Terms: React.FC = () => (
  <div className="max-w-2xl mx-auto p-6">
    <h1 className="text-2xl font-bold mb-4">Terms of Service</h1>
    <ul className="list-disc ml-5 space-y-2">
      <li>
        <b>Intermediary Status:</b> RoomShout is an <b>Intermediary</b> under
        Section 79 of the IT Act, 2000. We are not liable for user-generated
        content.
      </li>
      <li>
        <b>Prohibited Content:</b> You may not post content that is defamatory,
        obscene, invasive of privacy, promotes money laundering, or threatens
        the unity/integrity of India.
      </li>
      <li>
        <b>WhatsApp Handoff:</b> RoomShout does not verify landlords and is not
        a "Real Estate Agent" under RERA. We are only a communication link.
      </li>
    </ul>
  </div>
);

export default Terms;
