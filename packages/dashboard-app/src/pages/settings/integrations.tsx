import { SectionHeading, UserPrefView } from "@/components/preference/list";
import StatusCheck from "@/components/installs/statusCheck";
import settings from "@/data/integrations";
import installChecks from "@/data/install";

export default function Page() {
  return (
    <UserPrefView array={settings}>
      <section className="flex flex-col py-4">
        <SectionHeading index={settings.length}>Setup</SectionHeading>
        <div className="flex flex-col">
          {installChecks.map((check) => (
            <StatusCheck check={check} key={check.id} />
          ))}
        </div>
      </section>
    </UserPrefView>
  );
}
