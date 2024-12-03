import React, {useState} from "react";
import {ExtensionState} from "@/types";
import {Badge, Button, Card, Stack} from "react-bootstrap";
import Image from "next/image";
import defaultExtensionImg from "../../../public/icons/extension_icon.png";
import {ModMetadata, WrappedMod} from "@/components/mod/mods_search";
import {ExtensionButton} from "@/components/extension/extension_card";
import {invoke} from "@tauri-apps/api/core";

export const ModCard: React.FC<{
    onclick: (state: ExtensionState) => void,
    mod: WrappedMod
}> = ({
          onclick, mod
      }) => {
    let [state, setState] = useState(mod.state)

    const maxDescLength = 200

    let description = mod.metadata.description
    if (description.length > maxDescLength) {
        description = description.substring(0, maxDescLength - 3) + "..."
    }

    return <Card style={{
        margin: "10px 0",
        // backgroundColor: "rgba(55,37,42,255)"
    }} className="flex-row">
        <Card.Body style={{
            padding: "10px",
            lineHeight: 1
        }}>
            <Stack direction="horizontal" gap={3}>
                <Image
                    style={{
                        margin: "auto 0",
                        borderRadius: "10px"
                    }}
                    alt={""}
                    src={mod.metadata.icon_url ?? defaultExtensionImg}
                    height={70}
                    width={70}
                />
                <div>
                    <Card.Title as="h1" className="h1 h1-sm">
                        <Badge pill bg="primary" style={{
                            fontSize: "15px",
                            marginRight: "10px",
                            verticalAlign: "middle"
                        }}>
                            Mod
                        </Badge>
                        {mod.metadata.title}
                    </Card.Title>
                    {
                        mod.metadata.author ? `By ${mod.metadata.author}` : ""
                    }
                </div>

            </Stack>

            <Card.Text style={{
                margin: "25px 0"
            }}>{description}</Card.Text>
            <ExtensionButton state={state} onclick={(newState) => {
                setState(newState)
                onclick(newState)
            }}/>
            <Button
                style={{margin: "0 10px"}}
                variant="link"
                onClick={() => {
                    invoke("open_url",{
                        "url": ("https://modrinth.com/mod/" + (mod.metadata.project_id ?? mod.metadata.id))
                    }).then(() => {}).catch(() => {})
                }}
            >Open in browser</Button>
        </Card.Body>
    </Card>
}