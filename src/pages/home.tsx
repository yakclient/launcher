import LaunchLayout from "@/components/launch_layout";
import News from "@/components/news_page";
import Extensions from "@/components/extensions_search";
import Mods from "@/components/mods_page";
import Installed from "@/components/installed";
import Market from "@/components/market";

export default function Home() {
    return (
        <LaunchLayout
            pages={[
                {
                    name: "News",
                    content: <News/>
                },
                {
                    name: "Installed",
                    content: <Installed/>
                },
                {
                    name: "Market",
                    content: <Market/>
                },
            ]}
        />
    );
}
