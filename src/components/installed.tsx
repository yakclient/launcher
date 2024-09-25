import React, {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";
import {ExtensionMetadata, ExtensionPointer, ExtensionState, WrappedExtension} from "@/types";
import {Card} from "react-bootstrap";
import ExtensionCard from "@/components/extension_card";

const Installed: React.FC = () => {
    let [extensions, setExtensions] = useState<{
        metadata: ExtensionMetadata,
        pointer: ExtensionPointer
    }[]>([])

    let setupExtensions = async () => {
        let appliedExtensions = await invoke("get_extension_state") as ExtensionPointer[]
        console.log(appliedExtensions)

        let extension_metadata = await Promise.all(appliedExtensions.map(async (pointer) => {
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
        }))

        setExtensions(extension_metadata)
    }

    useEffect(() => {
       setupExtensions().then(() => {
           // Nothing
       })
    }, [])

    return <>
        {extensions.map((extension: WrappedExtension, index) => {
            return <ExtensionCard
                extension={extension}
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
        })}
    </>
}

export default Installed;