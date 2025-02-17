'use client'

import Image from "next/image";
import mc_png from "../../public/icons/mc_png.png"
import styles from "./launch_layout.module.css"
import React, {LegacyRef, MouseEventHandler, useEffect, useState} from "react";
import {Alert, Button, ButtonGroup, Container, Dropdown, Modal} from "react-bootstrap";
import BackgroundGradient from "@/components/bg_gradient";
import {Channel, invoke} from "@tauri-apps/api/core";
import {Alerts, ConsoleLine, useConsole} from "@/pages/_app";
import {useRouter} from "next/router";
import Nav from "@/components/nav";
import {listen} from "@tauri-apps/api/event";
import Settings from "@/components/settings/settings_popup";

// eslint-disable-next-line react/display-name
const ProfileButton = React.forwardRef((
    {onClick, uuid}: { onClick: MouseEventHandler, uuid: string },
    ref: LegacyRef<HTMLImageElement>,
) => (
    <Image
        ref={ref}
        alt={"Your profile"}
        onClick={(e) => {
            e.preventDefault()
            onClick(e)
        }}
        width={50}
        height={50}
        src={`https://mc-heads.net/avatar/${uuid}.png`}
    />

));

interface TaskEvent {
    name: string,
    id: number,
}

const LaunchLayout: React.FC<{
    pages: { name: string; content: React.ReactNode; }[],
}> = ({pages}) => {
    let [page, setPage] = useState(0)
    let [version, setVersion] = useState<string | null>(null)
    let [uuid, setUuid] = useState("")
    let [settingsOpen, setSettingsOpen] = useState(false)

    const console = useConsole()
    const router = useRouter();

    const versions = [
        "1.21.4", "1.21.3", "1.8.9"
    ]

    useEffect(() => {
        invoke("get_mc_profile")
            .then((it) => {
                setUuid((it as { id: string }).id)
            })
    }, [])

    return (
        <Alerts.Consumer>
            {alert =>
                <div className={styles.main}>
                    <Modal
                        show={settingsOpen}
                        onHide={() => setSettingsOpen(false)}
                        id={styles.settings_modal}
                    >
                        <Settings/>
                    </Modal>
                    <div
                        id={styles.profile}
                    >
                        <Dropdown>
                            <Dropdown.Toggle
                                as={ProfileButton}
                                uuid={uuid}>
                            </Dropdown.Toggle>

                            <Dropdown.Menu>
                                <Dropdown.Item
                                    onClick={() => {
                                        invoke("logout").then(() => {})
                                        router.push("/authentication").then(r => {})
                                    }}
                                >Logout</Dropdown.Item>
                                <Dropdown.Item onClick={() => {
                                    setSettingsOpen(true)
                                }}>Settings</Dropdown.Item>
                            </Dropdown.Menu>
                        </Dropdown>
                    </div>
                    <BackgroundGradient/>
                    <Image
                        src={mc_png}
                        alt={"Alt pic"}
                        // width={500}
                        height={281}
                        className={styles.title_image}
                    />
                    <Image
                        src={mc_png}
                        alt={"Alt pic"}
                        // width={500}
                        height={281}
                        className={styles.blurred_title_image}
                    />
                    <div className={styles.title}>
                        <div>
                            <span>YakClient</span>
                            <span>BETA2</span>
                        </div>
                    </div>
                    <Container id={styles.main_button_container} style={{
                        position: "relative",
                        zIndex: 2,
                    }}>
                        <Dropdown as={ButtonGroup} size="lg">
                            <Button
                                disabled={version == null}
                                variant="success"
                                onClick={() => {
                                    let channel = new Channel<ConsoleLine>();
                                    console.setChannel(channel)
                                    invoke("launch_minecraft", {
                                        version: version,
                                        consoleChannel: channel
                                    }).catch((it) => {
                                        alert(
                                            "danger",
                                            <>
                                                <Alert.Heading>Client error</Alert.Heading>
                                                <hr/>
                                                {it.toString()}
                                            </>
                                        )
                                    }).then(() => {
                                        router.push("/console")
                                    })
                                }}
                            >
                                Launch {version == null ? "" : version}
                            </Button>

                            <Dropdown.Toggle split variant="success" id="dropdown-split-basic"/>

                            <Dropdown.Menu style={{
                                overflow: "scroll",
                                maxHeight: "20em",
                            }}>
                                {
                                    versions.map((v, index) => <Dropdown.Item key={index} onClick={() => {
                                        setVersion(v)
                                    }}>{v}</Dropdown.Item>)
                                }
                            </Dropdown.Menu>
                        </Dropdown>
                    </Container>
                    <div id={styles.layout}>
                        <Nav
                            color={"#ff6347"}
                            fontSize={"20px"}
                            elements={pages.map(({name}) => {
                                return {
                                    name: name
                                }
                            })}
                            onChange={(index) => {
                                setPage(index)
                            }}
                        />
                        {pages[page].content}
                    </div>
                </div>
            }
        </Alerts.Consumer>
    );
}

export default LaunchLayout;