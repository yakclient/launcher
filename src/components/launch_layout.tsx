'use client'

import Image from "next/image";
import mc_png from "../public/icons/mc_png.png"
import styles from "./launch_layout.module.css"
import React, {useState} from "react";
import {Alert, Button, ButtonGroup, Col, Container, Dropdown, Row} from "react-bootstrap";
import BackgroundGradient from "@/components/bg_gradient";
import {invoke} from "@tauri-apps/api/tauri";
import {Alerts} from "@/pages/_app";
import {useRouter} from "next/router";

const NavBar: React.FC<{
    elements: { name: string; }[],
    onChange: (index: number) => void
}> = ({elements, onChange}) => {
    const [currIndex, setIndex] = useState(0);

    return <div id={styles.nav}>
        <Row>
            {elements.map(({name}, index) => (
                <Col className={`d-flex justify-content-center align-items-center ${styles.selector}`} onClick={() => {
                    if (index != currIndex) onChange(index)
                    setIndex(index)
                }} key={index}>
                    <span>
                        {name}
                    </span>
                </Col>
            ))}
        </Row>

        <div
            id={styles.nav_indicator}
            style={{
                width: ((100 / (elements.length)) + "%"),
                left: ((100 / (elements.length)) * currIndex) + "%"
            }}
        ></div>
    </div>
}

const LaunchLayout: React.FC<{
    pages: { name: string; content: React.ReactNode; }[],
}> = ({pages}) => {
    let [page, setPage] = useState(0)
    let [version, setVersion] = useState<string | null>(null)

    const router = useRouter();

    const versions = [
        "1.21", "1.20", "1.19"
    ]

    return (
        <Alerts.Consumer>
            {alert =>

                <div className={styles.main}>
                    <div
                        id={styles.relogin}
                    >
                        <Button
                            as={"a"}
                            href={"/"}
                            variant='outline-success'
                        >
                            Re Login
                        </Button>
                    </div>
                    <BackgroundGradient/>
                    <Image
                        src={mc_png}
                        alt={"Alt pic"}
                        width={500}
                        height={281}
                        className={styles.title_image}
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
                                    invoke("launch_minecraft", {
                                        version: version
                                    }).catch(() => {
                                        alert(
                                            "danger",
                                            <>
                                                <Alert.Heading>Client error</Alert.Heading>
                                                <hr/>
                                                A client error has occurred.
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
                        <NavBar
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