import React, {useState} from "react";
import styles from "@/components/nav.module.scss";
import {Col, Row} from "react-bootstrap";
import {fontDir} from "@tauri-apps/api/path";

const Nav: React.FC<{
    elements: { name: string; }[],
    onChange: (index: number) => void,
    color: string,
    fontSize: number,

}> = ({elements, onChange, color, fontSize}) => {
    const [currIndex, setIndex] = useState(0);
    return <div id={styles.nav}
                style={{
                    fontSize: fontSize ?? "20px"
                }
                }
    >
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
                left: ((100 / (elements.length)) * currIndex) + "%",
                backgroundColor: color ?? "#ff6347"
            }}
        ></div>
    </div>
}

export default Nav