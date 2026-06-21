import legal from "../data/legal.json";
import LegalPage from "../components/LegalPage.jsx";

export default function PrivacyPage() {
  return <LegalPage title="Privacy Policy" doc={legal.privacy} />;
}
