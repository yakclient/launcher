import React, {useContext, useEffect, useState} from "react";
import {Alert, Button, Form, InputGroup, ListGroup, Modal} from "react-bootstrap";
import {Alerts} from "@/pages/_app";
import {invoke} from "@tauri-apps/api/core";
import {
    ExtensionMetadata,
    ExtensionPointer,
    ExtensionState,
    ManagedExtensionMetadata, RepositoryType,
    SearchResult,
    WrappedExtension
} from "@/types";
import ExtensionCard from "@/components/extension/extension_card";
import SkeletonExtensionCard from "@/components/extension/skeleton_extension_card";

const queryServer = async (server: string, query: string, page: number = 0, pagination: number = 20): Promise<WrappedExtension[]> => {
    const url = `${server}/search?query=${encodeURIComponent(query)}&page=${page}&pagination=${pagination}`;

    let result = (await (await fetch(url)).json()) as SearchResult;

    let managedMetadata = await Promise.all(result.result.map(async (identifier) => {
        const metadataQuery = `${server}/metadata/${identifier.group.replace('.', '/')}/${identifier.name}`;

        let it1 = await fetch(metadataQuery);
        let it2 = await it1.json();
        return {
            metadata: it2 as ManagedExtensionMetadata,
            id: identifier
        };
    }))

    let metadata = await Promise.all(managedMetadata.map(async (managed) => {
        let release = managed.metadata.latest.release ?? managed.metadata.latest.rc ?? managed.metadata.latest.beta
        if (!release) {
            throw new Error("No Release found!")
        }

        const metadataQuery = `${server}/registry/` +
            `${managed.id.group.replaceAll('.', '/')}/` +
            `${managed.id.name}/` +
            `${release}/${managed.id.name}-${release}-metadata.json`;

        let it1 = await fetch(metadataQuery);
        let it2 = await it1.json();
        return {
            metadata: it2 as ExtensionMetadata,
            pointer: {
                descriptor: managed.id.group + ":" + managed.id.name + ":" + release,
                repository: server + "/registry",
                repository_type: RepositoryType.REMOTE
            } as ExtensionPointer
        };
    }))

    let appliedExtensions = new Set((await invoke("get_extension_state") as ExtensionPointer[])
        .map((t) => t.descriptor))

    return metadata
        .map((it) => {
            return {
                metadata: it.metadata,
                state: appliedExtensions.has(it.pointer.descriptor) ? ExtensionState.Enabled : ExtensionState.Disabled,
                pointer: it.pointer
            } as WrappedExtension
        })
}

const ExtensionSearch: React.FC = () => {
    const [extensions, setExtensions] = useState<WrappedExtension[]>([])
    const [searchTarget, setSearchTarget] = useState("");
    const [repositories, setRepositories] = useState<string[]>([
        "https://repo.extframework.dev"
    ])
    const [modalOpen, setModalOpen] = useState(false);
    const [repositoryInputTarget, setRepositoryInputTarget] = useState("");
    const [queryingServer, setQueryingServer] = useState(false)
    const addAlert = useContext(Alerts)

    useEffect(() => {
        setQueryingServer(true)
        queryServer("https://repo.extframework.dev", "")
            .then((res) => {
                setQueryingServer(false)
                setExtensions(res)
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
    }, [])

    let setupExtensions = (): React.ReactNode => {
        if (queryingServer) {
            return <SkeletonExtensionCard/>
        } else if (extensions.length == 0) {
            return <div style={{
                margin: "20px 0"
            }}>Nothing found</div>
        } else {
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
    }

    return <>
        <form onSubmit={(it) => {
            it.preventDefault()
            setQueryingServer(true)
            Promise.all(repositories.map((repo) => {
                return queryServer(repo, searchTarget)
            })).then((res) => {
                setQueryingServer(false)
                let flatMap = res.flatMap((it) => it)
                setExtensions(flatMap)
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
                <Form.Control placeholder={"Search"} onChange={(it) => {
                    setSearchTarget(it.target.value)
                }} value={searchTarget}/>
                <Button variant="outline-secondary" id="button-addon1" onClick={() => {
                    setModalOpen(true)
                }}>
                    Repositories
                </Button>
            </InputGroup>
            <Form.Text muted>
                Changes will take place on launch (installing, etc)
            </Form.Text>
        </form>
        <Modal
            show={modalOpen}
            onHide={() => {
                setModalOpen(false)
            }}>
            <Modal.Header closeButton>
                <Modal.Title>Configure Repositories</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <ListGroup as="ul">
                    {
                        repositories.map((it, i) => {
                            return <ListGroup.Item key={i}
                                                   className="d-flex justify-content-between align-items-center">
                                {it}
                                <Button variant="danger" size="sm" onClick={() => {
                                    setRepositories((prev) =>
                                        prev.filter((r) => r != it)
                                    )
                                }}>Remove</Button>
                            </ListGroup.Item>
                        })
                    }
                </ListGroup>
                <InputGroup className="mb-3" style={{
                    margin: "10px 0"
                }}>
                    <Form.Control value={repositoryInputTarget} onChange={(it) => {
                        setRepositoryInputTarget(it.target.value)
                    }}/>
                    <Button variant="outline-success" id="button-addon1" onClick={() => {
                        if (repositoryInputTarget == "") return

                        setRepositories([
                            ...repositories,
                            repositoryInputTarget
                        ])
                        setRepositoryInputTarget("")
                    }}>
                        Add
                    </Button>
                </InputGroup>
            </Modal.Body>
        </Modal>
        {
            setupExtensions()
        }
    </>
}

export default ExtensionSearch;