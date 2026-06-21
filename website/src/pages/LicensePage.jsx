import legal from "../data/legal.json";
import LegalPage from "../components/LegalPage.jsx";

export default function LicensePage() {
  return <LegalPage title="License" doc={legal.license} />;
}
