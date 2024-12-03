import React, {useContext, useEffect, useState} from "react";
import {Alert, Form, InputGroup} from "react-bootstrap";
import {Alerts} from "@/pages/_app";
import {ExtensionState} from "@/types";
import SkeletonExtensionCard from "@/components/extension/skeleton_extension_card";
import {invoke} from "@tauri-apps/api/core";
import {ModCard} from "@/components/mod/mod_card";
import {app} from "@tauri-apps/api";

type ModrinthSearchResult = {
    hits: ModMetadata[]
}

export type ModPointer = {
    project_id: string,
    loader: string
}

export type ModMetadata = {
    project_id: string,
    id: string,
    author: string,
    versions: string[],
    title: string,
    description: string,
    icon_url: string | null,
    categories: string[] | null
}

export type WrappedMod = {
    metadata: ModMetadata,
    state: ExtensionState,
    pointer: ModPointer,
}

const queryServer = async (query: string): Promise<WrappedMod[]> => {
    const url = `https://api.modrinth.com/v2/search?`
        + `query=${encodeURIComponent(query)}`
        + `&facets=${encodeURIComponent(`[["project_type:mod"],["categories:fabric"]]`)}`
        // + `&index=downloads`;

    const {hits} = await (await fetch(url)).json() as ModrinthSearchResult

    let enabledMods = new Set((await invoke("get_mod_state") as ModPointer[])
        .map((t) => t.project_id))

    return hits.map((hit) => {
        return {
            metadata: hit,
            state: enabledMods.has(hit.project_id) ? ExtensionState.Enabled : ExtensionState.Disabled,
            pointer: {
                project_id: hit.project_id,
                loader: "fabric"
            },
        }
    })
}

const Mods: React.FC = () => {
    const [mods, setMods] = useState<WrappedMod[]>([])
    const [searchTarget, setSearchTarget] = useState("");
    const [queryingServer, setQueryingServer] = useState(true)
    const addAlert = useContext(Alerts)

    useEffect(() => {
        queryServer("").then((hits) => {
            setQueryingServer(false)
            setMods(hits)
        }).catch((res) => {
            setQueryingServer(false)
            addAlert(
                "danger",
                <>
                    <Alert.Heading>Failed to search!</Alert.Heading>
                    <hr/>
                    {res.toString()}
                </>)
        })
    }, [])

    let setupMods = (): React.ReactNode => {
        if (queryingServer) {
            return <SkeletonExtensionCard/>
        } else if (mods.length == 0) {
            return <div style={{
                margin: "20px 0"
            }}>Nothing found</div>
        } else {
            return <>
                {mods.map((mod: WrappedMod, index) => {
                    return <ModCard
                        mod={mod}
                        onclick={async (state) => {
                            let currMods = await invoke("get_mod_state") as ModPointer[];

                            let appliedMods = state == ExtensionState.Disabled ?
                                currMods.filter((it) => {
                                    return it.project_id != mod.metadata.project_id
                                }) : ([...currMods, {
                                    project_id: mod.metadata.project_id,
                                    loader: mod.pointer.loader
                                }])

                            await invoke("set_mod_state", {
                                updated: appliedMods
                            })
                        }}
                        key={index}
                    />
                })}
            </>
        }
    }

    return <>
        {/*<Alerts.Consumer>*/}
        {/*    {addAlert =>*/}
                <>
                    <form onSubmit={(it) => {
                        it.preventDefault()
                        setQueryingServer(true)

                        // Promise.all(repositories.map((repo) => {
                        //     return queryServer(repo, searchTarget)
                        // })).then((res) => {
                        //     setQueryingServer(false)
                        //     let flatMap = res.flatMap((it) => it)
                        //     setExtensions(flatMap)
                        // })
                        queryServer(searchTarget)
                            .then((hits) => {
                                setQueryingServer(false)
                                setMods(hits)
                            })
                            .catch((res) => {
                                setQueryingServer(false)
                                addAlert(
                                    "danger",
                                    <>
                                        <Alert.Heading>Failed to search!</Alert.Heading>
                                        <hr/>
                                        {res.toString()}
                                    </>
                                )
                            })
                    }}>
                        <Form.Label>Search</Form.Label>
                        <InputGroup className="mb-3">
                            <Form.Control placeholder={"Search Modrinth"} onChange={(it) => {
                                setSearchTarget(it.target.value)
                            }} value={searchTarget}/>
                        </InputGroup>
                        <Form.Text muted>
                            Changes will take place on launch (installing, etc)
                        </Form.Text>
                    </form>
                    {
                        setupMods()
                    }
                </>
            {/*}*/}
        {/*</Alerts.Consumer>*/}
    </>
}

export default Mods