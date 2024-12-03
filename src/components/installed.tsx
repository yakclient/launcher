import React, {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {ExtensionMetadata, ExtensionPointer, ExtensionState, WrappedExtension} from "@/types";
import ExtensionCard from "@/components/extension/extension_card";
import metadata from "next/dist/server/typescript/rules/metadata";
import SkeletonExtensionCard from "@/components/extension/skeleton_extension_card";
import {ModMetadata, ModPointer, WrappedMod} from "@/components/mod/mods_search";
import {ModCard} from "@/components/mod/mod_card";

const Installed: React.FC = () => {
    const [queryingServer, setQueryingServer] = useState(false)
    let [extensions, setExtensions] = useState<{
        metadata: ExtensionMetadata,
        pointer: ExtensionPointer
    }[]>([])
    let [mods, setMods] = useState<{
        metadata: ModMetadata,
        pointer: ModPointer
    }[]>([])

    let setupCards = async () => {
        let appliedExtensions = await invoke("get_extension_state") as ExtensionPointer[]
        let enabledMods = await invoke("get_mod_state") as ModPointer[]

        setQueryingServer(true)
       Promise.all(appliedExtensions.map(async (pointer) => {
            let [group, name, version] = pointer.descriptor.split(":")

            const metadataQuery = `${pointer.repository}/registry/` +
                `${group.replaceAll('.', '/')}/` +
                `${name}/` +
                `${version}/${name}-${version}-metadata.json`;

            let it1 = await fetch(metadataQuery);
            let it2 = await it1.json();
            return {
                metadata: it2 as ExtensionMetadata,
                state: ExtensionState.Enabled,
                pointer: pointer
            } as WrappedExtension;
        })).then((extension_metadata) => {
           setQueryingServer(false)
           setExtensions(extension_metadata)
        })


        Promise.all(enabledMods.map(async (pointer) => {
            const metadataQuery = `https://api.modrinth.com/v2/project/${pointer.project_id}`;

            let it1 = await fetch(metadataQuery);
            let it2 = await it1.json();
            return {
                metadata: it2 as ModMetadata,
                state: ExtensionState.Enabled,
                pointer: pointer
            } as WrappedMod;
        })).then((mod_metadata) => {
            setQueryingServer(false)
            setMods(mod_metadata)
        })
    }

    useEffect(() => {
        setupCards().then(() => {
            // Nothing
        })
    }, [])

    let getCards = () => {
        if (queryingServer) {
            return <SkeletonExtensionCard/>
        } else {
            let extensionCards = extensions.map((extension, index) =>
                <ExtensionCard
                    extension={
                        {
                            metadata: extension.metadata,
                            pointer: extension.pointer,
                            state: ExtensionState.Enabled
                        } as WrappedExtension
                    }
                    onclick={async (state) => {
                        let currExtensions = await invoke("get_extension_state") as ExtensionPointer[];

                        let appliedExtensions = state == ExtensionState.Disabled ?
                            currExtensions.filter((it) => {
                                return it.descriptor != extension.pointer.descriptor
                            }) : ([...currExtensions, {
                                descriptor: extension.pointer.descriptor,
                                repository: extension.pointer.repository
                            }])

                        await invoke("set_extension_state", {
                            updated: appliedExtensions
                        })
                    }}
                    key={index}
                />
            );

            let modCards = mods.map(({metadata, pointer}, i) => {
                return <ModCard
                    mod={
                        {
                            metadata: metadata,
                            pointer: pointer,
                            state: ExtensionState.Enabled
                        } as WrappedMod
                    }
                    onclick={async (state) => {
                        let currMods = await invoke("get_mod_state") as ModPointer[];

                        let appliedMods = state == ExtensionState.Disabled ?
                            currMods.filter((it) => {
                                return it.project_id != pointer.project_id
                            }) : ([...currMods, {
                                project_id: pointer.project_id,
                            }])

                        await invoke("set_mod_state", {
                            updated: appliedMods
                        })
                    }}
                    key={i}
                />
            })

            return [
                ...extensionCards,
                ...modCards
            ]
        }
    }

    return getCards()
}

export default Installed;