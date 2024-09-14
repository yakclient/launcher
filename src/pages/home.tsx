import LaunchLayout from "@/components/launch_layout";
import News from "@/components/news_page";
import Extensions from "@/components/extensions_page";
import Mods from "@/components/mods_page";
import {useRouter} from "next/router";

export default function Home() {
    return (
        <LaunchLayout
            pages={[
                {
                    name: "News",
                    content: <News/>
                },
                {
                    name: "Extensions",
                    content: <Extensions/>
                },
                {
                    name: "Mods",
                    content: <Mods/>
                },
            ]}
        />
    );
}
