'use client'

import Image from "next/image";
import mc_png from "../../public/icons/mc_png.png"
import styles from "./launch_layout.module.css"
import React, {useEffect, useState} from "react";
import {Alert, Badge, Button, ButtonGroup, Col, Container, Dropdown, Row} from "react-bootstrap";
import BackgroundGradient from "@/components/bg_gradient";
import {Channel, invoke} from "@tauri-apps/api/core";
import {Alerts, ConsoleChannel, ConsoleLine, useConsole} from "@/pages/_app";
import {useRouter} from "next/router";
import Nav from "@/components/nav";

const ProfileButton = React.forwardRef(({ onClick, uuid }, ref) => (
    <Image
        alt={"Your profile"}
        onClick={(e) => {
            e.preventDefault()
            onClick(e)
        }}
        ref={ref}
        width={50}
        height={50}
        src={`https://mc-heads.net/avatar/${uuid}.png`}
    />
));

const LaunchLayout: React.FC<{
    pages: { name: string; content: React.ReactNode; }[],
}> = ({pages}) => {
    let [page, setPage] = useState(0)
    let [version, setVersion] = useState<string | null>(null)
    let [uuid, setUuid] = useState("")

    const router = useRouter();
    const console = useConsole()

    const versions = [
        "1.21.3", "1.21.2", "1.21.1", "1.8.9"
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
                    <div
                        id={styles.profile}
                    >
                        <Dropdown>
                            <Dropdown.Toggle
                                as={ProfileButton}
                                uuid={uuid}>
                            </Dropdown.Toggle>

                            <Dropdown.Menu>
                                <Dropdown.Item href={"/"}>Logout</Dropdown.Item>
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
                        className={styles.blured_title_image}
                    />
                    <div className={styles.title}>
                        <div>
                            <span>YakClient</span>
                            <span>BETA1</span>
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
                                    })
                                    router.push("/console")
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