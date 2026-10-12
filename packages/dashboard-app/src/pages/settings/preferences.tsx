import { SectionHeading, UserPrefView } from "@/components/preference/list";
import { Link } from "@/components/ui/link";
import settings from "@/data/preferences";

export default function Page() {
  return (
    <UserPrefView array={settings}>
      <section className="flex flex-col gap-4 py-4 order-last">
        <SectionHeading index={settings.length}>Licenses</SectionHeading>
        <Link to="/licenses" variant="blue" className="self-start text-sm">
          View licenses
        </Link>
      </section>
    </UserPrefView>
  );
}
