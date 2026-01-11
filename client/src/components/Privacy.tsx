import React from "react";

const Privacy: React.FC = () => (
  <div className="max-w-2xl mx-auto p-6">
    <h1 className="text-2xl font-bold mb-4">Privacy Policy</h1>
    <p className="mb-2">
      <b>Notice & Consent:</b> By using RoomShout, you (the Data Principal)
      consent to the collection of your IP address and browser fingerprint
      strictly for security purposes (fraud prevention).
    </p>
    <p className="mb-2">
      <b>Data Minimization & Erasure:</b> All posts and associated data are
      automatically deleted after 48 hours, in line with the DPDP Act, 2023 and
      our commitment to storage limitation.
    </p>
    <p className="mb-2">
      <b>Data Localization:</b> All personal data is stored on servers compliant
      with Indian data localization norms.
    </p>
  </div>
);

export default Privacy;
