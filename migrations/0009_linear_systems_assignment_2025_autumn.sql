SELECT insert_assignment_with_groups(
    'Тест',
    'test_assignment',
    '[
        {"name": "МКН-21БО", "deadline": "2025-12-24"},
        {"name": "Ф-21БО",   "deadline": "2025-12-24"},
        {"name": "РТ-21БО",   "deadline": "2025-12-24"},
        {"name": "ИТС-21БО",   "deadline": "2025-12-24"},
        {"name": "ИТС-22БО",   "deadline": "2025-12-24"}
    ]'
);

SELECT insert_assignment_with_groups(
    'Линейные уравнения и системы с постоянными коэффициентами',
    'linear_systems_2025',
    '[
        {"name": "МКН-21БО", "deadline": "2026-01-24"},
        {"name": "Ф-21БО",   "deadline": "2026-01-24"},
        {"name": "РТ-21БО",   "deadline": "2026-01-24"},
        {"name": "ИТС-21БО",   "deadline": "2026-01-24"},
        {"name": "ИТС-22БО",   "deadline": "2026-01-24"}
    ]'
);
